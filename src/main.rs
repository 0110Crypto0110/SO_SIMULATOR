use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex; // Importa Mutex do tokio
use csv::Writer;
use eframe::{self, egui};
use egui::ViewportBuilder;

mod paciente;
mod recursos;
mod monitor_gui;
// mod sincronizacao; // REMOVIDO: Arquivo obsoleto
mod estatisticas;
mod monitor;

use paciente::Paciente;
use recursos::{Recursos, HistoricoUso, EventoUso};
use monitor_gui::{MonitorGUI, EstadoRecursosGUI};
use estatisticas::Estatisticas;
use monitor::iniciar_monitor;

use egui_plot::{Plot, BarChart, Bar, PlotPoint};

const ESCALA_TEMPO: f64 = 5.0;
const QTD_MEDICOS: usize = 3;
const QTD_SALAS: usize = 2;
const QTD_LEITOS: usize = 4;
// Constante para Exames, mantida aqui, mas não usada nas chamadas de construtor abaixo
const QTD_EXAMES: usize = 4; 

#[derive(Debug, Clone)]
pub struct Snapshot {
    tempo: f64,
    medicos: usize,
    salas: usize,
    leitos: usize,
    // Nota: O campo 'exames' está faltando aqui, mantendo a estrutura original fornecida.
}

// Estrutura para rastrear o histórico de recursos disponíveis
#[derive(Debug)]
pub struct HistoricoRecursos {
    snapshots: Vec<Snapshot>,
    inicio: std::time::Instant,
}

impl HistoricoRecursos {
    pub fn new() -> Self {
        Self {
            snapshots: vec![],
            inicio: std::time::Instant::now(),
        }
    }

    /// Registra o estado atual dos recursos no histórico.
    /// `recursos` contém a contagem de semáforos disponíveis (atômico).
    pub fn registrar(&mut self, recursos: &Recursos) {
        let t = self.inicio.elapsed().as_secs_f64();
        self.snapshots.push(Snapshot {
            tempo: (t * 100.0).round() / 100.0,
            medicos: recursos.medicos.available_permits(),
            salas: recursos.salas_cirurgia.available_permits(),
            leitos: recursos.leitos.available_permits(),
        });
    }
}

/// Salva o histórico de snapshots de recursos em um arquivo CSV.
async fn salvar_historico_csv(historico: &Arc<Mutex<HistoricoRecursos>>, filename: &str) -> Result<(), csv::Error> {
    let historico_lock = historico.lock().await;
    let mut wtr = Writer::from_path(filename)?;
    
    wtr.write_record(&["tempo", "medicos_disp", "salas_disp", "leitos_disp"])?;
    for snap in &historico_lock.snapshots {
        wtr.serialize((snap.tempo, snap.medicos, snap.salas, snap.leitos))?;
    }
    wtr.flush()?;
    println!("✅ Histórico de recursos salvo em: {}", filename);
    Ok(())
}

/// Salva os logs da simulação em um arquivo CSV.
async fn salvar_logs_csv(logs: &Arc<Mutex<Vec<String>>>, filename: &str) -> Result<(), csv::Error> {
    let logs_lock = logs.lock().await;
    let mut wtr = Writer::from_path(filename)?;
    
    wtr.write_record(&["indice", "mensagem"])?;
    let mut i = 1;
    for log in logs_lock.iter() {
        wtr.serialize((i, log))?;
        i += 1;
    }
    wtr.flush()?;
    println!("✅ Logs de simulação salvos em: {}", filename);
    Ok(())
}

/// Função utilitária para registrar logs no Mutex de logs de forma assíncrona.
async fn registrar_log(logs: &Arc<Mutex<Vec<String>>>, mensagem: &str) {
    logs.lock().await.push(mensagem.to_string());
}

/// Estrutura de GUI para exibir logs em tempo real.
struct LogGUI {
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogGUI {
    fn new(logs: Arc<Mutex<Vec<String>>>) -> Self {
        Self { logs }
    }
}

impl eframe::App for LogGUI {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // Uso de try_lock() para acessar o tokio::sync::Mutex de forma não-bloqueante na thread da GUI.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Logs de Execução em Tempo Real");
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    
                    if let Ok(logs) = self.logs.try_lock() {
                        for log in logs.iter() {
                            let mut texto = egui::RichText::new(log);
                            if log.contains("✅") {
                                texto = texto.color(egui::Color32::GREEN);
                            } else if log.contains("❌") || log.contains("⚠️") || log.contains("Deadlock Contornado") {
                                // Deadlock Contornado agora é vermelho para dar destaque
                                texto = texto.color(egui::Color32::RED).strong(); 
                            } else if log.contains("🕓") {
                                texto = texto.color(egui::Color32::LIGHT_BLUE);
                            }
                            ui.label(texto);
                        }
                    } else {
                        ui.label(egui::RichText::new("Aguardando logs...").color(egui::Color32::YELLOW));
                    }
                });
        });
        ctx.request_repaint();
    }
}

/// Estrutura de GUI para exibir gráficos e estatísticas da simulação.
struct GraficoApp {
    historico_recursos: Arc<Mutex<HistoricoRecursos>>,
    historico_uso: Arc<HistoricoUso>, // Contém Mutexes de Tokio internamente
    estatisticas: Arc<Estatisticas>, // Contém Mutexes de Std
}

impl GraficoApp {
    fn new(
        historico_recursos: Arc<Mutex<HistoricoRecursos>>,
        historico_uso: Arc<HistoricoUso>,
        estatisticas: Arc<Estatisticas>,
    ) -> Self {
        Self {
            historico_recursos,
            historico_uso,
            estatisticas,
        }
    }

    /// Desenha o gráfico de ocupação de recursos ao longo do tempo.
    fn mostrar_grafico_ocupacao(&self, ui: &mut egui::Ui, max_time: f64) {
        // Usamos .blocking_lock() na thread da GUI para acessar o tokio::sync::Mutex de HistoricoUso
        let medico_eventos = self.historico_uso.medico.blocking_lock().clone();
        let sala_eventos = self.historico_uso.sala.blocking_lock().clone();
        let leito_eventos = self.historico_uso.leito.blocking_lock().clone();

        let plot = Plot::new("timeline_recursos")
            .width(ui.available_width())
            .height(500.0)
            .include_x(0.0)
            .include_x(max_time * 1.05)
            .label_formatter(|_, val| format!("{:.1}s", val.x))
            .allow_zoom(true)
            .allow_drag(true)
            .center_y_axis(false)
            .show_x(true)
            .show_y(false);

        plot.show(ui, |plot_ui| {
            let cores = [
                egui::Color32::from_rgb(46, 139, 87),
                egui::Color32::from_rgb(255, 99, 71),
                egui::Color32::from_rgb(60, 179, 113),
                egui::Color32::from_rgb(255, 165, 0),
                egui::Color32::from_rgb(0, 191, 255),
                egui::Color32::from_rgb(147, 112, 219),
                egui::Color32::from_rgb(240, 128, 128),
                egui::Color32::from_rgb(240, 128, 128),
            ];
            let mut cor_idx = 0;
            let mut y_offset = 0.0;
            let bar_height = 0.4;

            let mut plotar_eventos = |current_y_offset: f64, cor_idx_ref: &mut usize, nome_recurso: &str, eventos: &Vec<EventoUso>| -> f64 {
                if eventos.is_empty() {
                    return 0.0;
                }

                plot_ui.text(
                    egui_plot::Text::new(
                        PlotPoint::new(max_time * 0.05, current_y_offset + 0.5),
                        nome_recurso,
                    )
                    .anchor(egui::Align2::LEFT_CENTER)
                    .color(egui::Color32::BLACK),
                );

                let max_id = eventos.iter().map(|e| e.instancia_id).max().unwrap_or(1);
                let mut bars = Vec::new();

                for evento in eventos {
                    let duracao = if evento.fim == 0.0 {
                        max_time - evento.inicio
                    } else {
                        evento.fim - evento.inicio
                    };
                    let centro_x = evento.inicio + duracao / 2.0;
                    let y = current_y_offset + evento.instancia_id as f64 * 0.5;

                    let bar = Bar::new(centro_x, y)
                        .width(duracao)
                        .name(format!(
                            "{}: {}",
                            nome_recurso.split(" ").next().unwrap_or(""),
                            evento.nome_paciente.clone()
                        ))
                        .fill(cores[*cor_idx_ref % cores.len()])
                        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK));

                    bars.push(bar);
                    *cor_idx_ref += 1;

                    plot_ui.text(
                        egui_plot::Text::new(
                            PlotPoint::new(0.0, y),
                            format!("{}", evento.instancia_id),
                        )
                        .anchor(egui::Align2::LEFT_CENTER)
                        .color(egui::Color32::DARK_GRAY),
                    );
                }

                plot_ui.bar_chart(BarChart::new(bars).width(bar_height));
                (max_id as f64 * 0.5) + 0.5
            };

            y_offset += plotar_eventos(y_offset, &mut cor_idx, "MÉDICOS", &medico_eventos);
            y_offset += 0.5;
            y_offset += plotar_eventos(y_offset, &mut cor_idx, "SALAS", &sala_eventos);
            y_offset += 0.5;
            y_offset += plotar_eventos(y_offset, &mut cor_idx, "LEITOS", &leito_eventos);

            plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                [0.0, -1.0],
                [max_time * 1.05, y_offset + 1.0],
            ));
        });
    }
}

impl eframe::App for GraficoApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut max_time = 0.0;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("📊 Estatísticas e Gráfico de Ocupação");
                ui.separator();

                ui.heading("Resumo da Simulação");
                
                // Usando try_lock() no Mutex do tokio para o histórico
                if let Ok(historico) = self.historico_recursos.try_lock() {
                    max_time = historico.snapshots.last().map(|s| s.tempo).unwrap_or(0.0);
                    ui.label(format!("Tempo Total de Simulação: {:.2}s", max_time));
                } else {
                    ui.label(egui::RichText::new("Aguardando dados de tempo...").color(egui::Color32::YELLOW));
                }

                let atendimentos = self.estatisticas.obter_atendimentos();
                // O Mutex dentro de Estatisticas é std::sync::Mutex, usamos .lock().unwrap()
                let lock = atendimentos.lock().unwrap();
                let total: f64 = lock.values().map(|d| d.as_secs_f64()).sum();
                let total_concluidos = lock.len();
                let media = if total_concluidos > 0 {
                    total / total_concluidos as f64
                } else {
                    0.0
                };
                ui.label(format!("Total de Pacientes Atendidos: {}", total_concluidos));
                ui.label(format!("Tempo médio de atendimento: {:.2}s", media));
                drop(lock);

                ui.separator();

                // Apenas mostra o gráfico se tiver um tempo base (max_time > 0.0)
                if max_time > 0.0 {
                    ui.heading("Linha do Tempo de Uso de Recursos (Quem usou quando)");
                    self.mostrar_grafico_ocupacao(ui, max_time);
                } else {
                    ui.label("Gráfico aguardando dados de simulação...".to_string());
                }
            });
        });
        ctx.request_repaint();
    }
}

/// Função principal que inicializa o runtime do Tokio, o estado compartilhado e as GUIs Eframe.
fn main() -> eframe::Result<()> {
    // 1. Inicializa o runtime do Tokio
    let rt = Runtime::new().unwrap();

    // 2. Inicializa recursos e estados compartilhados usando Arc<tokio::sync::Mutex<...>>
    let recursos = Arc::new(Recursos::novo(QTD_MEDICOS, QTD_SALAS, QTD_LEITOS, QTD_EXAMES));
    
    // Todos os dados acessados em tarefas assíncronas usam tokio::sync::Mutex
    let estado_gui = Arc::new(Mutex::new(EstadoRecursosGUI::new(QTD_MEDICOS, QTD_SALAS, QTD_LEITOS, QTD_EXAMES)));
    let historico = Arc::new(Mutex::new(HistoricoRecursos::new()));
    let estatisticas = Arc::new(Estatisticas::novo());
    let logs = Arc::new(Mutex::new(vec![]));

    let pacientes_simulacao = vec![
        Paciente::novo_com_prioridade("P01-Critico", 45, "Infarto", true, 1),
        Paciente::novo_com_prioridade("P02-Normal", 22, "Fratura", false, 0),
        Paciente::novo_com_prioridade("P03-Normal", 70, "Apendicite", true, 0),
        Paciente::novo_com_prioridade("P04-Critico", 30, "AVC", false, 1),
        Paciente::novo_com_prioridade("P05-Normal", 55, "Gripe Forte", false, 0),
        Paciente::novo_com_prioridade("P06-Normal", 18, "Corte", false, 0),
        Paciente::novo_com_prioridade("P07-Critico", 60, "Politraum", true, 1),
        Paciente::novo_com_prioridade("P08-Normal", 35, "Dor", false, 0),
    ];

    let mut handles = vec![];

    // 3. Spawna a tarefa do Monitor
    handles.push(rt.spawn(iniciar_monitor(
        recursos.clone(),
        estado_gui.clone(),
        historico.clone(),
        logs.clone(),
    )));

    let recursos_clone = recursos.clone();
    let estado_clone = estado_gui.clone();
    let logs_clone = logs.clone();
    let historico_clone = historico.clone();
    let estatisticas_pac = estatisticas.clone();

    // 4. Spawna a tarefa de simulação dos pacientes
    handles.push(rt.spawn(async move {
        registrar_log(&logs_clone, "Sistema iniciado. Iniciando atendimento...").await;
        
        // Spawna uma tarefa para cada paciente
        let handles: Vec<_> = pacientes_simulacao
            .into_iter()
            .map(|paciente| {
                let recursos_pac = recursos_clone.clone();
                let estado_pac = estado_clone.clone();
                let logs_clone_pac = logs_clone.clone();
                let estatisticas_pac_clone = estatisticas_pac.clone();

                tokio::spawn(async move {
                    registrar_log(
                        &logs_clone_pac,
                        &format!("🔹 Paciente {} iniciou o processo de atendimento.", paciente.nome),
                    ).await;
                    
                    let inicio = estatisticas_pac_clone.iniciar_atendimento(&paciente.nome);
                    
                    // Onde a simulação do paciente acontece
                    let concluido_sucesso = paciente.atender_com_escala(recursos_pac, estado_pac, ESCALA_TEMPO).await;
                    
                    estatisticas_pac_clone.finalizar_atendimento(&paciente.nome, inicio, concluido_sucesso); 
                    
                    if concluido_sucesso {
                        registrar_log(
                            &logs_clone_pac,
                            &format!("✅ Concluído atendimento: {}", paciente.nome),
                        ).await;
                    }
                })
            })
            .collect();

        // Aguarda a conclusão de todos os pacientes
        for handle in handles {
            let _ = handle.await;
        }

        // Gera o relatório final
        estatisticas_pac.imprimir_relatorio(); 

        // Dá um pequeno tempo para o monitor registrar o último estado
        sleep(Duration::from_secs_f64(ESCALA_TEMPO * 0.5)).await;
        
        registrar_log(&logs_clone, "🏁 Simulação concluída.").await;

        // Salva os dados
        let _ = salvar_historico_csv(&historico_clone, "historico_recursos.csv").await;
        let _ = salvar_logs_csv(&logs_clone, "logs_simulacao.csv").await;
    }));

    // 5. Roda as tarefas assíncronas do Tokio em uma thread dedicada
    std::thread::spawn(move || {
        rt.block_on(async {
            for handle in handles {
                let _ = handle.await;
            }
        });
    });

    // 6. Roda as GUIs Eframe em threads separadas, incluindo a thread principal

    // MonitorGUI (na thread principal)
    let monitor_app = MonitorGUI::new(estado_gui.clone(), recursos.clone());
    let options_monitor = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(egui::vec2(650.0, 500.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Monitor de Recursos (Slots)",
        options_monitor,
        Box::new(|_cc| Ok(Box::new(monitor_app))),
    ).expect("Falha ao rodar o MonitorGUI");


    // LogGUI (em thread separada)
    let logs_clone_gui = logs.clone();
    std::thread::spawn(move || {
        let logs_app = LogGUI::new(logs_clone_gui);
        let options = eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(egui::vec2(600.0, 400.0)),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "Logs de Execução",
            options,
            Box::new(|_cc| Ok(Box::new(logs_app))),
        );
    });

    // GraficoApp (em thread separada)
    let historico_graf = historico.clone();
    let historico_uso_graf = recursos.historico_uso.clone();
    let estatisticas_graf = estatisticas.clone();
    std::thread::spawn(move || {
        let graf_app = GraficoApp::new(historico_graf, historico_uso_graf, estatisticas_graf);
        let options = eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(egui::vec2(1000.0, 700.0)),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "Estatísticas e Gráfico de Uso",
            options,
            Box::new(|_cc| Ok(Box::new(graf_app))),
        );
    });

    // O retorno Ok(()) é necessário, mas o eframe::run_native acima já bloqueia.
    // Retornamos Ok(()) para satisfazer a assinatura.
    Ok(())
}
