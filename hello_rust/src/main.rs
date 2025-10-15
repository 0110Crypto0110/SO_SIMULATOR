use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};
use csv::Writer;

mod paciente;
mod recursos;
mod monitor_gui;
mod sincronizacao;
mod estatisticas;
mod monitor;

use paciente::Paciente;
use recursos::Recursos;
use monitor_gui::{MonitorGUI, EstadoRecursosGUI};
use estatisticas::Estatisticas;
use monitor::iniciar_monitor;

const ESCALA_TEMPO: f64 = 5.0;

#[derive(Debug)]
struct Snapshot {
    tempo: f64,
    medicos: usize,
    salas: usize,
    leitos: usize,
}

#[derive(Debug)]
struct HistoricoRecursos {
    snapshots: Vec<Snapshot>,
    inicio: std::time::Instant,
}

impl HistoricoRecursos {
    fn new() -> Self {
        Self {
            snapshots: vec![],
            inicio: std::time::Instant::now(),
        }
    }

    fn registrar(&mut self, recursos: &Recursos) {
        let t = self.inicio.elapsed().as_secs_f64();
        self.snapshots.push(Snapshot {
            tempo: (t * 100.0).round() / 100.0,
            medicos: recursos.medicos.available_permits(),
            salas: recursos.salas_cirurgia.available_permits(),
            leitos: recursos.leitos.available_permits(),
        });
    }
}

fn salvar_historico_csv(historico: &Arc<Mutex<HistoricoRecursos>>, path: &str) -> std::io::Result<()> {
    let historico = historico.lock().unwrap();
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(&["tempo", "medicos_disponiveis", "salas_disponiveis", "leitos_disponiveis"])?;
    for snap in &historico.snapshots {
        wtr.write_record(&[
            snap.tempo.to_string(),
            snap.medicos.to_string(),
            snap.salas.to_string(),
            snap.leitos.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

fn salvar_logs_csv(logs: &Arc<Mutex<Vec<String>>>, path: &str) -> std::io::Result<()> {
    let logs = logs.lock().unwrap();
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(&["log"])?;
    for log in logs.iter() {
        wtr.write_record(&[log])?;
    }
    wtr.flush()?;
    Ok(())
}

/// GUI de logs
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📝 Logs de Execução");
            let logs = self.logs.lock().unwrap();
            for log in logs.iter() {
                ui.label(log);
            }
        });
        ctx.request_repaint();
    }
}

/// GUI de gráficos e estatísticas
struct GraficoApp {
    historico: Arc<Mutex<HistoricoRecursos>>,
    estatisticas: Arc<Estatisticas>,
}

impl GraficoApp {
    fn new(historico: Arc<Mutex<HistoricoRecursos>>, estatisticas: Arc<Estatisticas>) -> Self {
        Self { historico, estatisticas }
    }
}

impl eframe::App for GraficoApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📊 Histórico de Recursos");

            let historico = self.historico.lock().unwrap();
            if let Some(last) = historico.snapshots.last() {
                ui.label(format!(
                    "Tempo: {:.2}s | Médicos: {} | Salas: {} | Leitos: {}",
                    last.tempo, last.medicos, last.salas, last.leitos
                ));
            }

            let atendimentos = self.estatisticas.obter_atendimentos();
            let lock = atendimentos.lock().unwrap();
            let total: f64 = lock.values().map(|d| d.as_secs_f64()).sum();
            let media = if lock.len() > 0 { total / lock.len() as f64 } else { 0.0 };
            ui.label(format!("Tempo médio de atendimento: {:.2}s", media));
        });
        ctx.request_repaint();
    }
}

/// Função auxiliar para registrar logs
fn registrar_log(logs: &Arc<Mutex<Vec<String>>>, msg: &str) {
    let mut l = logs.lock().unwrap();
    l.push(msg.to_string());
}

/// Main
fn main() -> eframe::Result<()> {
    let recursos = Arc::new(Recursos::novo(3, 2, 5));
    let estado_gui = Arc::new(Mutex::new(EstadoRecursosGUI::new(3, 2, 5)));
    let historico = Arc::new(Mutex::new(HistoricoRecursos::new()));
    let logs = Arc::new(Mutex::new(vec![]));
    let estatisticas = Arc::new(Estatisticas::novo());

    let rt = Runtime::new().unwrap();

    // Spawn simulação
    {
        let recursos_clone = recursos.clone();
        let estado_clone = estado_gui.clone();
        let historico_clone = historico.clone();
        let logs_clone = logs.clone();
        let estatisticas_clone = estatisticas.clone();

        rt.spawn(async move {
            iniciar_monitor(recursos_clone.clone(), estado_clone.clone()).await;

            let mut pacientes = vec![];
            for i in 1..=50 {
                let nome = format!("Paciente_{}", i);
                let idade = 20 + (i % 50);
                let condicao = if i % 5 == 0 { "Crítico" } else { "Normal" };
                let cirurgia = i % 3 == 0;
                let prioridade = if condicao == "Crítico" { 1 } else { 0 };
                pacientes.push(Paciente::novo_com_prioridade(
                    &nome, idade, &condicao, cirurgia, prioridade
                ));
            }

            // Histórico periódico
            let recursos_hist = recursos_clone.clone();
            let historico_hist = historico_clone.clone();
            tokio::spawn(async move {
                loop {
                    {
                        let mut h = historico_hist.lock().unwrap();
                        h.registrar(&recursos_hist);
                    }
                    sleep(Duration::from_millis(500)).await;
                }
            });

            // Atendimentos
            let mut handles = vec![];
            for paciente in pacientes {
                let recursos_pac = recursos_clone.clone();
                let estado_gui_pac = estado_clone.clone();
                let logs_clone_pac = logs_clone.clone();
                let estatisticas_pac = estatisticas_clone.clone();

                handles.push(tokio::spawn(async move {
                    registrar_log(&logs_clone_pac, &format!("🔹 Iniciando atendimento: {}", paciente.nome));
                    let inicio = estatisticas_pac.iniciar_atendimento(&paciente.nome);

                    paciente.atender_com_escala(recursos_pac, estado_gui_pac, ESCALA_TEMPO).await;

                    estatisticas_pac.finalizar_atendimento(&paciente.nome, inicio);
                    registrar_log(&logs_clone_pac, &format!("✅ Concluído atendimento: {}", paciente.nome));
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }

            // Salvar CSVs
            let _ = salvar_historico_csv(&historico_clone, "historico_recursos.csv");
            let _ = salvar_logs_csv(&logs_clone, "logs_simulacao.csv");
        });
    }

    // GUI Monitor
    let monitor_app = MonitorGUI::new(estado_gui.clone(), recursos.clone());
    let options = eframe::NativeOptions::default();
    eframe::run_native("Monitor de Recursos", options.clone(), Box::new(|_cc| Ok(Box::new(monitor_app))))?;

    // GUI de logs
    let logs_clone_gui = logs.clone();
    std::thread::spawn(move || {
        let logs_app = LogGUI::new(logs_clone_gui);
        let options = eframe::NativeOptions::default();
        eframe::run_native("Logs de Execução", options, Box::new(|_cc| Ok(Box::new(logs_app))));
    });

    // GUI do gráfico
    let historico_graf = historico.clone();
    let estatisticas_graf = estatisticas.clone();
    std::thread::spawn(move || {
        let graf_app = GraficoApp::new(historico_graf, estatisticas_graf);
        let options = eframe::NativeOptions::default();
        eframe::run_native("Gráfico em Tempo Real", options, Box::new(|_cc| Ok(Box::new(graf_app))));
    });

    Ok(())
}
