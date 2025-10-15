use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use crate::recursos::Recursos;

// Imports corretos do egui_plot
use egui_plot::{Plot, Line, Legend, Corner, PlotPoints};

pub struct EstadoRecursosGUI {
    pub medicos: usize,
    pub medicos_max: usize,
    pub salas: usize,
    pub salas_max: usize,
    pub leitos: usize,
    pub leitos_max: usize,
    pub fila_medicos: Vec<String>,
    pub fila_salas: Vec<String>,
    pub fila_leitos: Vec<String>,
    pub deadlock_medicos: Vec<String>,
    pub deadlock_salas: Vec<String>,
    pub deadlock_leitos: Vec<String>,
    pub atendimentos_total: u32,
    pub pacientes_atendidos: Vec<String>,
    pub tempos_espera: Vec<(String, Duration)>,
    pub progresso_pacientes: HashMap<String, f32>,
    pub logs: Vec<String>,
}

impl EstadoRecursosGUI {
    pub fn new(medicos_max: usize, salas_max: usize, leitos_max: usize) -> Self {
        Self {
            medicos: medicos_max,
            medicos_max,
            salas: salas_max,
            salas_max,
            leitos: leitos_max,
            leitos_max,
            fila_medicos: vec![],
            fila_salas: vec![],
            fila_leitos: vec![],
            deadlock_medicos: vec![],
            deadlock_salas: vec![],
            deadlock_leitos: vec![],
            atendimentos_total: 0,
            pacientes_atendidos: vec![],
            tempos_espera: vec![],
            progresso_pacientes: HashMap::new(),
            logs: vec![],
        }
    }

    pub fn atualizar_estado(&mut self, recursos: &Recursos) {
        self.medicos = recursos.medicos.available_permits();
        self.salas = recursos.salas_cirurgia.available_permits();
        self.leitos = recursos.leitos.available_permits();
        self.fila_medicos = recursos.fila_medicos.lock().unwrap().clone();
        self.fila_salas = recursos.fila_salas.lock().unwrap().clone();
        self.fila_leitos = recursos.fila_leitos.lock().unwrap().clone();
        self.deadlock_medicos = recursos.deadlock_medicos.lock().unwrap().clone();
        self.deadlock_salas = recursos.deadlock_salas.lock().unwrap().clone();
        self.deadlock_leitos = recursos.deadlock_leitos.lock().unwrap().clone();
    }

    pub fn atualizar_progresso(&mut self, paciente: &str, progresso: f32) {
        self.progresso_pacientes.insert(paciente.to_string(), progresso);
        self.logs.push(format!("Progresso {}: {:.1}%", paciente, progresso * 100.0));
    }

    pub fn registrar_atendimento(&mut self, paciente: &str, duracao: Duration) {
        self.pacientes_atendidos.push(paciente.to_string());
        self.atendimentos_total += 1;
        self.tempos_espera.push((paciente.to_string(), duracao));
        self.logs.push(format!("✅ Concluído atendimento: {}", paciente));
    }

    pub fn registrar_log(&mut self, mensagem: &str) {
        self.logs.push(mensagem.to_string());
    }
}

pub struct MonitorGUI {
    estado: Arc<Mutex<EstadoRecursosGUI>>,
    recursos: Arc<Recursos>,
    blink_start: Instant,
    historico: Arc<Mutex<Vec<(f64, usize, usize, usize)>>>, // (tempo, medicos, salas, leitos)
    inicio: Instant,
}

impl MonitorGUI {
    pub fn new(estado: Arc<Mutex<EstadoRecursosGUI>>, recursos: Arc<Recursos>) -> Self {
        Self {
            estado,
            recursos,
            blink_start: Instant::now(),
            historico: Arc::new(Mutex::new(vec![])),
            inicio: Instant::now(),
        }
    }

    fn blink(&self) -> bool {
        (self.blink_start.elapsed().as_millis() / 500) % 2 == 0
    }

    fn registrar_historico(&self) {
        let estado = self.estado.lock().unwrap();
        let tempo = self.inicio.elapsed().as_secs_f64();
        let snapshot = (tempo, estado.medicos, estado.salas, estado.leitos);
        self.historico.lock().unwrap().push(snapshot);
    }

    fn mostrar_recursos(&self, ui: &mut egui::Ui) {
        let estado = self.estado.lock().unwrap();
        let recursos_vec = vec![
            ("👨‍⚕️ Médicos", estado.medicos, estado.medicos_max, &estado.fila_medicos, &estado.deadlock_medicos),
            ("🏥 Salas Cirurgia", estado.salas, estado.salas_max, &estado.fila_salas, &estado.deadlock_salas),
            ("🛏️ Leitos", estado.leitos, estado.leitos_max, &estado.fila_leitos, &estado.deadlock_leitos),
        ];

        let max_scroll_height = ui.available_height() * 0.15;

        for (nome, disponiveis, max, fila, deadlock) in recursos_vec {
            let percentual = if max == 0 { 0.0 } else { disponiveis as f32 / max as f32 };
            let color = match percentual {
                p if p < 0.3 => egui::Color32::RED,
                p if p < 0.6 => egui::Color32::YELLOW,
                _ => egui::Color32::GREEN,
            };

            ui.horizontal(|ui| {
                ui.label(nome);
                let largura = ui.available_width() * 0.6;
                ui.add_sized([largura, 20.0], egui::ProgressBar::new(percentual).fill(color).show_percentage());
                ui.label(format!("{}/{}", disponiveis, max));
            });

            if !fila.is_empty() || !deadlock.is_empty() {
                egui::ScrollArea::vertical()
                    .id_source(nome)
                    .max_height(max_scroll_height)
                    .show(ui, |ui| {
                        for paciente in fila {
                            let mut texto = egui::RichText::new(paciente);
                            if paciente.contains("Maria") || paciente.contains("Ana") {
                                texto = texto.color(egui::Color32::RED).strong();
                            }
                            if deadlock.contains(paciente) && self.blink() {
                                texto = texto.background_color(egui::Color32::from_rgb(255,100,100)).strong();
                            }
                            ui.label(texto);
                        }
                        if !deadlock.is_empty() {
                            ui.label(
                                egui::RichText::new("⚠️ Deadlock detectado")
                                    .color(egui::Color32::RED)
                                    .strong()
                            );
                        }
                    });
            }
            ui.separator();
        }
    }

    fn mostrar_progresso(&self, ui: &mut egui::Ui) {
        let estado = self.estado.lock().unwrap();
        if estado.progresso_pacientes.is_empty() { return; }
        egui::ScrollArea::vertical()
            .id_source("scroll_progresso")
            .show(ui, |ui| {
                for (paciente, progresso) in &estado.progresso_pacientes {
                    ui.horizontal(|ui| {
                        ui.label(paciente);
                        let largura = ui.available_width() * 0.6;
                        ui.add_sized([largura, 18.0], egui::ProgressBar::new(*progresso).show_percentage());
                    });
                }
            });
    }

    fn mostrar_logs(&self, ui: &mut egui::Ui) {
        let estado = self.estado.lock().unwrap();
        if estado.logs.is_empty() { return; }
        egui::ScrollArea::vertical()
            .id_source("scroll_logs")
            .show(ui, |ui| {
                for log in &estado.logs {
                    let mut texto = egui::RichText::new(log);
                    if log.contains("🔹") { texto = texto.color(egui::Color32::LIGHT_BLUE); }
                    else if log.contains("✅") { texto = texto.color(egui::Color32::GREEN); }
                    else if log.contains("⚠️") { texto = texto.color(egui::Color32::RED).strong(); }
                    ui.label(texto);
                }
            });
    }

    fn mostrar_graficos(&self, ui: &mut egui::Ui) {
        let historico = self.historico.lock().unwrap();
        if historico.is_empty() { return; }

        // Converter iteradores em Vec<[f64; 2]> para PlotPoints
        let dados_medicos: Vec<[f64; 2]> = historico.iter().map(|(t, m, _, _)| [*t, *m as f64]).collect();
        let dados_salas: Vec<[f64; 2]> = historico.iter().map(|(t, _, s, _)| [*t, *s as f64]).collect();
        let dados_leitos: Vec<[f64; 2]> = historico.iter().map(|(t, _, _, l)| [*t, *l as f64]).collect();

        let line_medicos = Line::new(dados_medicos).color(egui::Color32::BLUE).name("Médicos");
        let line_salas   = Line::new(dados_salas).color(egui::Color32::from_rgb(255,165,0)).name("Salas");
        let line_leitos  = Line::new(dados_leitos).color(egui::Color32::GREEN).name("Leitos");

        Plot::new("plot_recursos")
            .legend(Legend::default().position(Corner::RightTop))
            .show(ui, |plot_ui| {
                plot_ui.line(line_medicos);
                plot_ui.line(line_salas);
                plot_ui.line(line_leitos);
            });
    }
}

impl eframe::App for MonitorGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Atualiza o estado com os recursos atuais
        let mut estado = self.estado.lock().unwrap();
        estado.atualizar_estado(&self.recursos);
        drop(estado);

        // 🔹 Registrar histórico continuamente
        self.registrar_historico();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("🩺 Monitoramento Hospitalar");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::CollapsingHeader::new("Recursos").show(ui, |ui| {
                self.mostrar_recursos(ui);
            });

            egui::CollapsingHeader::new("Progresso Pacientes").show(ui, |ui| {
                self.mostrar_progresso(ui);
            });

            egui::CollapsingHeader::new("📊 Estatísticas e Gráficos").show(ui, |ui| {
                self.mostrar_graficos(ui);
            });

            egui::CollapsingHeader::new("📝 Logs").show(ui, |ui| {
                self.mostrar_logs(ui);
            });
        });

        ctx.request_repaint();
    }
}
