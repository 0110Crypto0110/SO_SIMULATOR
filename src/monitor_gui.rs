use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex; 
use std::sync::Mutex as StdMutex; 
use std::time::{Instant, Duration};
use std::collections::HashMap;
use crate::recursos::Recursos;
use egui_plot::{Plot, Line, Legend, Corner, PlotPoints};

/// Estrutura que espelha o estado dos recursos do sistema para uso síncrono na GUI.
///
/// **CORREÇÃO: Adicionados campos para o recurso 'Exames'.**
#[derive(Clone)]
pub struct EstadoRecursosGUI {
    pub medicos: usize,
    pub medicos_max: usize,
    pub salas: usize,
    pub salas_max: usize,
    pub leitos: usize,
    pub leitos_max: usize,
    // NOVO: Recursos de Exames
    pub exames: usize,
    pub exames_max: usize,
    
    pub medicos_em_uso_slots: Vec<Option<String>>,
    pub salas_em_uso_slots: Vec<Option<String>>,
    pub leitos_em_uso_slots: Vec<Option<String>>,
    // NOVO: Slots de Exames
    pub exames_em_uso_slots: Vec<Option<String>>,

    pub fila_medicos: Vec<String>,
    pub fila_salas: Vec<String>,
    pub fila_leitos: Vec<String>,
    // NOVO: Fila de Exames
    pub fila_exames: Vec<String>,

    pub deadlock_medicos: Vec<String>,
    pub deadlock_salas: Vec<String>,
    pub deadlock_leitos: Vec<String>,
    // NOVO: Deadlock de Exames
    pub deadlock_exames: Vec<String>,

    pub atendimentos_total: u32,
    pub pacientes_atendidos: Vec<String>,
    pub tempos_espera: Vec<(String, Duration)>,
    pub progresso_pacientes: HashMap<String, f32>,
    pub logs: Vec<String>,
}

impl EstadoRecursosGUI {
    /// Cria uma nova instância do estado da GUI.
    /// **CORREÇÃO: Adicionado 'exames_max' ao construtor.**
    pub fn new(medicos_max: usize, salas_max: usize, leitos_max: usize, exames_max: usize) -> Self {
        Self {
            medicos: medicos_max,
            medicos_max,
            salas: salas_max,
            salas_max,
            leitos: leitos_max,
            leitos_max,
            exames: exames_max, // NOVO
            exames_max,         // NOVO
            
            medicos_em_uso_slots: vec![None; medicos_max],
            salas_em_uso_slots: vec![None; salas_max],
            leitos_em_uso_slots: vec![None; leitos_max],
            exames_em_uso_slots: vec![None; exames_max], // NOVO

            fila_medicos: vec![],
            fila_salas: vec![],
            fila_leitos: vec![],
            fila_exames: vec![], // NOVO
            
            deadlock_medicos: vec![],
            deadlock_salas: vec![],
            deadlock_leitos: vec![],
            deadlock_exames: vec![], // NOVO

            atendimentos_total: 0,
            pacientes_atendidos: vec![],
            tempos_espera: vec![],
            progresso_pacientes: HashMap::new(),
            logs: vec![],
        }
    }

    /// Atualiza o estado da GUI a partir da estrutura de recursos principal.
    /// **CORREÇÃO: Incluída a lógica de atualização para os recursos de Exames.**
    pub fn atualizar_estado(&mut self, recursos: &Recursos) {
        // 1. Atualiza as contagens de permissões disponíveis
        self.medicos = recursos.medicos.available_permits();
        self.salas = recursos.salas_cirurgia.available_permits();
        self.leitos = recursos.leitos.available_permits();
        self.exames = recursos.equipamentos_exames.available_permits(); // NOVO: Assumindo nome 'equipamentos_exames'
        
        // 2. Copia o estado dos slots de uso
        self.medicos_em_uso_slots = recursos.slots_medicos.blocking_lock().clone();
        self.salas_em_uso_slots = recursos.slots_salas.blocking_lock().clone();
        self.leitos_em_uso_slots = recursos.slots_leitos.blocking_lock().clone();
        self.exames_em_uso_slots = recursos.slots_exames.blocking_lock().clone(); // NOVO: Assumindo nome 'slots_exames'

        // 3. Atualiza as filas de espera e deadlock
        // CORREÇÃO ESSENCIAL: Removemos a chamada `.into_iter().map(|t| t.1).collect()`
        // e simplesmente clonamos o conteúdo do Mutex, assumindo que ele já é Vec<String>
        // (que é o tipo de destino dos campos self.*).
        
        self.fila_medicos = recursos.fila_medicos.blocking_lock().clone();
        self.fila_salas = recursos.fila_salas.blocking_lock().clone();
        self.fila_leitos = recursos.fila_leitos.blocking_lock().clone();
        self.fila_exames = recursos.fila_exames.blocking_lock().clone();

        self.deadlock_medicos = recursos.deadlock_medicos.blocking_lock().clone();
        self.deadlock_salas = recursos.deadlock_salas.blocking_lock().clone();
        self.deadlock_leitos = recursos.deadlock_leitos.blocking_lock().clone();
        self.deadlock_exames = recursos.deadlock_exames.blocking_lock().clone();
    }

    pub fn atualizar_progresso(&mut self, paciente: &str, progresso: f32) {
        if progresso >= 1.0 {
            self.progresso_pacientes.remove(paciente);
        } else {
            self.progresso_pacientes.insert(paciente.to_string(), progresso);
        }
    }

    pub fn registrar_atendimento(&mut self, paciente: &str, duracao: Duration) {
        self.pacientes_atendidos.push(paciente.to_string());
        self.atendimentos_total += 1;
        self.tempos_espera.push((paciente.to_string(), duracao));
    }

    pub fn registrar_log(&mut self, mensagem: &str) {
        self.logs.push(mensagem.to_string());
        // Limita o histórico de logs para evitar estouro de memória
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }
}

pub struct MonitorGUI {
    estado: Arc<Mutex<EstadoRecursosGUI>>,
    recursos: Arc<Recursos>,
    blink_start: Instant,
    /// Histórico de uso de recursos para plotagem (Tempo, Médicos em Uso, Salas em Uso, Leitos em Uso, Exames em Uso).
    historico: Arc<StdMutex<Vec<(f64, usize, usize, usize, usize)>>>, // CORREÇÃO: Adicionado 'usize' para Exames
    inicio: Instant,
}

impl MonitorGUI {
    pub fn new(estado: Arc<Mutex<EstadoRecursosGUI>>, recursos: Arc<Recursos>) -> Self {
        Self {
            estado,
            recursos,
            blink_start: Instant::now(),
            historico: Arc::new(StdMutex::new(vec![])),
            inicio: Instant::now(),
        }
    }

    /// Lógica para determinar se o elemento deve piscar.
    fn blink(&self) -> bool {
        (self.blink_start.elapsed().as_millis() / 500) % 2 == 0
    }

    /// Desenha o painel de recursos, seus slots de uso e as filas de espera/deadlock.
    fn mostrar_recursos(&self, ui: &mut egui::Ui) {
        let estado = self.estado.blocking_lock();

        // CORREÇÃO: Adicionado o recurso Exames para exibição
        let recursos_info = vec![
            ("👨‍⚕️ Médicos", &estado.medicos_em_uso_slots, &estado.fila_medicos, &estado.deadlock_medicos),
            ("🏥 Salas Cirurgia", &estado.salas_em_uso_slots, &estado.fila_salas, &estado.deadlock_salas),
            ("🛏️ Leitos", &estado.leitos_em_uso_slots, &estado.fila_leitos, &estado.deadlock_leitos),
            ("🔬 Exames", &estado.exames_em_uso_slots, &estado.fila_exames, &estado.deadlock_exames), // NOVO
        ];

        let max_scroll_height = ui.available_height() * 0.15;

        // Usa quatro colunas para acomodar o novo recurso
        ui.columns(4, |columns| {
            for (col_idx, (nome_recurso, slots, fila, deadlock)) in recursos_info.into_iter().enumerate() {
                let ui = &mut columns[col_idx];
                ui.vertical(|ui| {
                    ui.heading(nome_recurso);
                    ui.separator();

                    // Exibição dos Slots de Uso
                    for (i, uso) in slots.iter().enumerate() {
                        let (texto, cor) = match uso {
                            Some(paciente) => (format!("Em uso: {}", paciente), egui::Color32::DARK_RED),
                            None => ("Disponível".to_string(), egui::Color32::DARK_GREEN),
                        };

                        ui.horizontal(|ui| {
                            let label_texto = format!("{} {}:", nome_recurso.split(" ").last().unwrap_or("Recurso"), i + 1);
                            ui.label(egui::RichText::new(label_texto).strong());
                            ui.label(egui::RichText::new(texto).color(cor));
                        });
                    }

                    ui.separator();

                    // Exibição de Filas e Deadlocks
                    if !fila.is_empty() || !deadlock.is_empty() {
                        ui.label(format!("Aguardando: {} | Deadlock: {}", fila.len(), deadlock.len()));
                        
                        let total_espera: Vec<&String> = fila.iter().chain(deadlock.iter()).collect();
                        
                        egui::ScrollArea::vertical()
                            .id_source(format!("fila_{}", nome_recurso))
                            .max_height(max_scroll_height)
                            .show(ui, |ui| {
                                for paciente in total_espera {
                                    let mut texto = egui::RichText::new(paciente);
                                    if deadlock.contains(paciente) {
                                        // Aplica efeito de piscar em deadlock
                                        texto = texto.color(egui::Color32::RED).strong();
                                        if self.blink() {
                                            texto = texto.background_color(egui::Color32::from_rgb(255, 100, 100));
                                        }
                                    }
                                    ui.label(texto);
                                }
                            });
                    } else {
                        ui.label("Fila vazia.");
                    }
                });
            }
        });
    }

    /// Desenha as barras de progresso dos pacientes em atendimento.
    fn mostrar_progresso(&self, ui: &mut egui::Ui) {
        let estado = self.estado.blocking_lock();
        
        ui.label(format!("Pacientes em atendimento: {}", estado.progresso_pacientes.len()));
        
        if estado.progresso_pacientes.is_empty() {
            ui.label("Nenhum paciente em atendimento.");
            return;
        }

        egui::ScrollArea::vertical()
            .id_source("scroll_progresso")
            .max_height(ui.available_height() * 0.5)
            .show(ui, |ui| {
                let mut pacientes_progresso: Vec<(&String, &f32)> = estado.progresso_pacientes.iter().collect();
                pacientes_progresso.sort_by_key(|a| a.0);
                for (paciente, progresso) in pacientes_progresso {
                    ui.horizontal(|ui| {
                        ui.label(paciente);
                        let largura = ui.available_width() * 0.8;
                        ui.add_sized([largura, 18.0], egui::ProgressBar::new(*progresso).show_percentage());
                    });
                }
            });
    }

    /// Desenha o histórico de logs de eventos do sistema.
    fn mostrar_logs(&self, ui: &mut egui::Ui) {
        let estado = self.estado.blocking_lock();
        
        if estado.logs.is_empty() {
            ui.label("Aguardando logs...");
            return;
        }

        egui::ScrollArea::vertical()
            .id_source("scroll_logs")
            .auto_shrink([false, false])
            .stick_to_bottom(true) // Garante que a barra de rolagem fique no final para novos logs
            .show(ui, |ui| {
                for log in &estado.logs {
                    let mut texto = egui::RichText::new(log);
                    // Colore o log com base no conteúdo
                    if log.contains("🔹") { // Ação
                        texto = texto.color(egui::Color32::LIGHT_BLUE);
                    } else if log.contains("✅") { // Sucesso
                        texto = texto.color(egui::Color32::GREEN);
                    } else if log.contains("⚠️") { // Aviso
                        texto = texto.color(egui::Color32::YELLOW);
                    } else if log.contains("❌") { // Erro/Deadlock
                        texto = texto.color(egui::Color32::RED).strong();
                    }
                    ui.label(texto);
                }
            });
    }

    /// Desenha o gráfico de linha de uso de recursos ao longo do tempo.
    fn mostrar_graficos(&self, ui: &mut egui::Ui) {
        let historico = self.historico.lock().unwrap();
        if historico.len() < 2 {
            ui.label("Sem dados de histórico suficientes para plotar (mínimo 2 pontos).");
            return;
        }

        // Os dados de histórico são (tempo_em_segundos, uso_medicos, uso_salas, uso_leitos, uso_exames)
        let dados_medicos: PlotPoints = historico.iter().map(|(t, m, _, _, _)| [*t, *m as f64]).collect();
        let dados_salas: PlotPoints = historico.iter().map(|(t, _, s, _, _)| [*t, *s as f64]).collect();
        let dados_leitos: PlotPoints = historico.iter().map(|(t, _, _, l, _)| [*t, *l as f64]).collect();
        let dados_exames: PlotPoints = historico.iter().map(|(t, _, _, _, e)| [*t, *e as f64]).collect(); // NOVO

        let line_medicos = Line::new(dados_medicos).color(egui::Color32::BLUE).name("Médicos em Uso");
        let line_salas = Line::new(dados_salas).color(egui::Color32::from_rgb(255, 165, 0)).name("Salas em Uso"); // Laranja
        let line_leitos = Line::new(dados_leitos).color(egui::Color32::GREEN).name("Leitos em Uso");
        let line_exames = Line::new(dados_exames).color(egui::Color32::from_rgb(128, 0, 128)).name("Exames em Uso"); // Roxo

        let estado = self.estado.blocking_lock();

        Plot::new("plot_recursos")
            .legend(Legend::default().position(Corner::RightTop))
            .include_y(0.0)
            // Inclui o valor máximo de cada recurso no eixo Y
            .include_y(estado.medicos_max as f64)
            .include_y(estado.salas_max as f64)
            .include_y(estado.leitos_max as f64)
            .include_y(estado.exames_max as f64) // NOVO
            .allow_drag(true)
            .allow_zoom(true)
            .label_formatter(|name, value| {
                if name.is_empty() {
                    // Formata o valor X (tempo) e Y (uso)
                    format!("Tempo: {:.1}s\nUso: {:.0}", value.x, value.y)
                } else {
                    format!("{} @ Tempo: {:.1}s\nUso: {:.0}", name, value.x, value.y)
                }
            })
            .show(ui, |plot_ui| {
                // CORREÇÃO: trocado .with_stroke() por .stroke()
                plot_ui.line(line_medicos.stroke(egui::Stroke::new(2.0, egui::Color32::BLUE)));
                plot_ui.line(line_salas.stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 165, 0))));
                plot_ui.line(line_leitos.stroke(egui::Stroke::new(2.0, egui::Color32::GREEN)));
                plot_ui.line(line_exames.stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(128, 0, 128)))); // NOVO
            });
    }
}

impl eframe::App for MonitorGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let elapsed = self.inicio.elapsed();
        let elapsed_secs = elapsed.as_secs();
        let elapsed_mins = elapsed_secs / 60;
        let elapsed_secs_remainder = elapsed_secs % 60;
        let elapsed_time_str = format!("Tempo de Simulação: {:02}m {:02}s", elapsed_mins, elapsed_secs_remainder);

        {
            let mut estado = self.estado.blocking_lock();
            // Atualiza o estado da GUI a partir da simulação
            estado.atualizar_estado(&self.recursos); 

            // Atualização do histórico (Uso = Max - Disponível)
            let m_uso = estado.medicos_max - estado.medicos;
            let s_uso = estado.salas_max - estado.salas;
            let l_uso = estado.leitos_max - estado.leitos;
            let e_uso = estado.exames_max - estado.exames; // NOVO

            // Adiciona ponto ao histórico a cada segundo
            let historico_len = self.historico.lock().unwrap().len();
            if historico_len == 0 || (self.inicio.elapsed().as_secs_f64() - self.historico.lock().unwrap().last().unwrap().0) >= 1.0 {
                self.historico.lock().unwrap().push((self.inicio.elapsed().as_secs_f64(), m_uso, s_uso, l_uso, e_uso)); // CORREÇÃO: Adicionado e_uso
            }

            // Remove pontos antigos se o histórico for muito longo (manter no máximo 300 segundos)
            self.historico.lock().unwrap().retain(|(t, _, _, _, _)| self.inicio.elapsed().as_secs_f64() - *t < 300.0);
        }

        // Lógica de piscar (reinicia a cada 500ms)
        if self.blink_start.elapsed() > Duration::from_millis(500) {
            self.blink_start = Instant::now();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🩺 Monitoramento Hospitalar - Simulação de Concorrência e Deadlock");
                ui.add_space(20.0);
                ui.label(egui::RichText::new(elapsed_time_str).strong().color(egui::Color32::WHITE));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Usa ScrollArea para que o conteúdo se ajuste a telas menores
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(5.0);

                // Painel de Recursos e Filas
                egui::CollapsingHeader::new("Recursos e Filas de Espera").default_open(true).show(ui, |ui| {
                    self.mostrar_recursos(ui);
                });

                ui.add_space(10.0);

                // Painel de Progresso
                egui::CollapsingHeader::new("Progresso Pacientes em Atendimento").default_open(true).show(ui, |ui| {
                    self.mostrar_progresso(ui);
                });

                ui.add_space(10.0);

                // Painel de Logs
                egui::CollapsingHeader::new("📝 Logs de Eventos").default_open(false).show(ui, |ui| {
                    // Restringe a altura do log
                    ui.set_max_height(ui.available_height() * 0.5);
                    self.mostrar_logs(ui);
                });

                ui.add_space(10.0);

                // Painel de Gráficos
                egui::CollapsingHeader::new("📊 Histórico de Uso de Recursos").show(ui, |ui| {
                    // Garante espaço suficiente para o gráfico
                    ui.set_min_height(300.0); 
                    self.mostrar_graficos(ui);
                });

                ui.add_space(10.0);
                
                // Exibe as estatísticas gerais no final
                let estado = self.estado.blocking_lock();
                ui.group(|ui| {
                    ui.heading("Estatísticas Gerais");
                    ui.label(format!("Atendimentos Concluídos: {}", estado.atendimentos_total));
                    
                    let total_espera: Duration = estado.tempos_espera.iter().map(|(_, d)| *d).sum();
                    let media_espera = if estado.atendimentos_total > 0 {
                        total_espera.as_secs_f64() / estado.atendimentos_total as f64
                    } else {
                        0.0
                    };
                    ui.label(format!("Tempo Médio de Espera (Concluídos): {:.2}s", media_espera));
                });
                
            });
        });

        // Solicita repintura contínua para atualizar o estado e o efeito de piscar
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
