use crate::paciente::Paciente;
use crate::recursos::Recursos;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::task;

/// Estrutura que mantém a fila de pacientes
pub struct Hospital {
    pub fila: Arc<Mutex<VecDeque<Arc<Paciente>>>>,
    pub recursos: Arc<Recursos>,
}

impl Hospital {
    pub fn novo(recursos: Arc<Recursos>) -> Self {
        Self {
            fila: Arc::new(Mutex::new(VecDeque::new())),
            recursos,
        }
    }

    /// Adiciona um paciente à fila
    pub fn adicionar_paciente(&self, paciente: Arc<Paciente>) {
        let mut fila = self.fila.lock().unwrap();
        fila.push_back(paciente);
    }

    /// Despacha pacientes da fila para atendimento
    pub fn iniciar_atendimento(&self, estado_gui: Arc<Mutex<EstadoRecursos>>) {
        let fila_clone = self.fila.clone();
        let recursos_clone = self.recursos.clone();

        tokio::spawn(async move {
            loop {
                let paciente_opt = {
                    let mut fila = fila_clone.lock().unwrap();
                    fila.pop_front()
                };

                if let Some(paciente) = paciente_opt {
                    let recursos_clone2 = recursos_clone.clone();
                    let estado_clone = estado_gui.clone();
                    tokio::spawn(async move {
                        paciente.atender(recursos_clone2, estado_clone).await;
                    });
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });
    }

}


