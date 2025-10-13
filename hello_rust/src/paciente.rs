use crate::recursos::Recursos;
use crate::sincronizacao::{esperar_recurso, usar_recurso, pausa};
use std::sync::Arc;


#[derive(Clone)]
pub struct Paciente {
    pub nome: String,
    pub idade: u32,
    pub condicao: String,
    pub precisa_cirurgia: bool,
    pub prioridade: u8, // prioridade no atendimento
}

impl Paciente {
    pub fn novo(nome: &str, idade: u32, condicao: &str, precisa_cirurgia: bool, prioridade: u8) -> Self {
        Paciente {
            nome: nome.to_string(),
            idade,
            condicao: condicao.to_string(),
            precisa_cirurgia,
            prioridade,
           
        }
    }

    pub async fn atender(&self, recursos: Arc<Recursos>) {
        println!("\n🔹 Iniciando atendimento de {} ({}) - {}", 
                 self.nome, self.idade, self.condicao);

        // Consulta (médico)
        let medico = esperar_recurso("Médico", &self.nome, recursos.clone()).await;
        usar_recurso(&format!("Consulta - {}", self.nome), 2).await;
        recursos.libera_recurso("Médico", &self.nome);
        drop(medico);

        // Exames (sem médico)
        usar_recurso(&format!("Exames - {}", self.nome), 2).await;

        // Cirurgia (médico + sala)
        if self.precisa_cirurgia {
            let medico = esperar_recurso("Médico", &self.nome, recursos.clone()).await;
            let sala = esperar_recurso("Sala de Cirurgia", &self.nome, recursos.clone()).await;
            usar_recurso(&format!("Cirurgia - {}", self.nome), 3).await;
            recursos.libera_recurso("Sala de Cirurgia", &self.nome);
            recursos.libera_recurso("Médico", &self.nome);
            drop(sala);
            drop(medico);

            // Leito após cirurgia
            let leito = esperar_recurso("Leito", &self.nome, recursos.clone()).await;
            usar_recurso(&format!("Leito - {}", self.nome), 3).await;
            recursos.libera_recurso("Leito", &self.nome);
            drop(leito);
        }

        pausa(1).await;
        println!("✅ {} recebeu alta médica.", self.nome);
    }
}
