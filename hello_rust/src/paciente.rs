use crate::recursos::Recursos;
use crate::sincronizacao::{usar_recurso, pausa};
use crate::monitor_gui::EstadoRecursosGUI;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Paciente {
    pub nome: String,
    pub idade: u32,
    pub condicao: String,
    pub precisa_cirurgia: bool,
    pub prioridade: u8, // 0 = normal, 1 = crítico
}

impl Paciente {
    /// Cria um paciente
    pub fn novo(nome: &str, idade: u32, condicao: &str, precisa_cirurgia: bool, prioridade: u8) -> Self {
        Paciente {
            nome: nome.to_string(),
            idade,
            condicao: condicao.to_string(),
            precisa_cirurgia,
            prioridade,
        }
    }
    

    /// Função compatível com os testes existentes
    pub fn novo_com_prioridade(nome: &str, idade: u32, condicao: &str, precisa_cirurgia: bool, prioridade: u8) -> Self {
        Self::novo(nome, idade, condicao, precisa_cirurgia, prioridade)
    }

    /// Atendimento do paciente, respeitando filas, prioridades e GUI
    pub async fn atender_com_escala(
        &self,
        recursos: Arc<Recursos>,
        estado_gui: Arc<Mutex<EstadoRecursosGUI>>,
        escala: f64, // 1.0 = tempo real, >1.0 = mais lento
    ) {
        println!("\n🔹 Iniciando atendimento de {} ({}) - {}", self.nome, self.idade, self.condicao);

        let etapas = if self.precisa_cirurgia { 4 } else { 2 };
        let mut progresso: f32 = 0.0;

        // -------------------- Consulta --------------------
        let inicio_consulta = Instant::now();
        {
            let mut estado = estado_gui.lock().unwrap();
            if self.prioridade == 1 {
                estado.fila_medicos.insert(0, self.nome.clone());
            } else {
                estado.fila_medicos.push(self.nome.clone());
            }
            estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
        }

        let medico = recursos.reservar_medico(self.nome.clone()).await;
        let espera_consulta = inicio_consulta.elapsed();
        {
            let mut estado = estado_gui.lock().unwrap();
            estado.fila_medicos.retain(|n| n != &self.nome);
            estado.registrar_atendimento(&self.nome, espera_consulta);
            progresso += 1.0;
            estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
        }

        usar_recurso(&format!("Consulta - {}", self.nome), (2.0 * escala) as u64).await;
        drop(medico);

        // -------------------- Exames --------------------
        usar_recurso(&format!("Exames - {}", self.nome), (2.0 * escala) as u64).await;
        {
            let mut estado = estado_gui.lock().unwrap();
            progresso += 1.0;
            estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
        }

        // -------------------- Cirurgia --------------------
        if self.precisa_cirurgia {
            let inicio_cirurgia = Instant::now();
            {
                let mut estado = estado_gui.lock().unwrap();
                if self.prioridade == 1 {
                    estado.fila_medicos.insert(0, self.nome.clone());
                    estado.fila_salas.insert(0, self.nome.clone());
                } else {
                    estado.fila_medicos.push(self.nome.clone());
                    estado.fila_salas.push(self.nome.clone());
                }
                estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
            }

            let medico = recursos.reservar_medico(self.nome.clone()).await;
            let sala = recursos.reservar_sala(self.nome.clone()).await;
            let espera_cirurgia = inicio_cirurgia.elapsed();
            {
                let mut estado = estado_gui.lock().unwrap();
                estado.fila_medicos.retain(|n| n != &self.nome);
                estado.fila_salas.retain(|n| n != &self.nome);
                estado.registrar_atendimento(&self.nome, espera_cirurgia);
                progresso += 1.0;
                estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
            }

            usar_recurso(&format!("Cirurgia - {}", self.nome), (3.0 * escala) as u64).await;
            drop(sala);
            drop(medico);

            // -------------------- Leito pós-cirurgia --------------------
            let inicio_leito = Instant::now();
            {
                let mut estado = estado_gui.lock().unwrap();
                if self.prioridade == 1 {
                    estado.fila_leitos.insert(0, self.nome.clone());
                } else {
                    estado.fila_leitos.push(self.nome.clone());
                }
                estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
            }

            let leito = recursos.reservar_leito(self.nome.clone()).await;
            let espera_leito = inicio_leito.elapsed();
            {
                let mut estado = estado_gui.lock().unwrap();
                estado.fila_leitos.retain(|n| n != &self.nome);
                estado.registrar_atendimento(&self.nome, espera_leito);
                progresso += 1.0;
                estado.atualizar_progresso(&self.nome, progresso / etapas as f32);
            }

            usar_recurso(&format!("Leito - {}", self.nome), (3.0 * escala) as u64).await;
            drop(leito);
        }

        pausa((1.0 * escala) as u64).await;
        println!("✅ {} recebeu alta médica.", self.nome);

        let mut estado = estado_gui.lock().unwrap();
        estado.atualizar_progresso(&self.nome, 1.0);
    }
}