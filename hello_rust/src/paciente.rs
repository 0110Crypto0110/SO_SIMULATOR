use crate::recursos::Recursos;
use crate::sincronizacao::{esperar_recurso, usar_recurso, pausa};
use std::sync::Arc;

pub struct Paciente {
    pub nome: String,
    pub idade: u32,
    pub condicao: String,
    pub precisa_cirurgia: bool,
}

impl Paciente {
    pub fn novo(nome: &str, idade: u32, condicao: &str, precisa_cirurgia: bool) -> Self {
        Paciente {
            nome: nome.to_string(),
            idade,
            condicao: condicao.to_string(),
            precisa_cirurgia,
        }
    }

    pub async fn atender(&self, recursos: Arc<Recursos>) {
        println!("\n🔹 Iniciando atendimento de {} ({}) - {}", 
                 self.nome, self.idade, self.condicao);

        // Consulta (precisa de médico)
        let medico = esperar_recurso("Médico", recursos.medicos.clone()).await;
        usar_recurso(&format!("Consulta - {}", self.nome), 2).await;
        drop(medico);

        // Exames (sem médico)
        usar_recurso(&format!("Exames - {}", self.nome), 2).await;

        // Cirurgia (médico + sala)
        if self.precisa_cirurgia {
            let medico = esperar_recurso("Médico", recursos.medicos.clone()).await;
            let sala = esperar_recurso("Sala de Cirurgia", recursos.salas_cirurgia.clone()).await;
            usar_recurso(&format!("Cirurgia - {}", self.nome), 3).await;
            drop(sala);
            drop(medico);

            // Leito após cirurgia
            let leito = esperar_recurso("Leito", recursos.leitos.clone()).await;
            usar_recurso(&format!("Leito - {}", self.nome), 3).await;
            drop(leito);
        }

        pausa(1).await;
        println!("✅ {} recebeu alta médica.", self.nome);
    }
}
