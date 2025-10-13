mod paciente;
mod recursos;
mod sincronizacao;
mod monitor;

use paciente::Paciente;
use recursos::Recursos;
use std::sync::Arc;
use tokio::{task, time::{sleep, Duration}};

#[tokio::main]
async fn main() {
    println!("\n🏥 Iniciando Simulador de Hospital — Projeto de SO\n");

    // Inicializa os recursos do hospital
    let recursos = Arc::new(Recursos::novo(5, 2, 10));

    // Inicia o monitoramento em uma task paralela
    let recursos_monitor = recursos.clone();
    task::spawn({
        let recursos_clone = recursos_monitor.clone();
        async move {
            monitor::iniciar_monitor(recursos_clone).await;
        }
    });

    // Criação os pacientes com diferentes condições
    let mut pacientes = vec![
        Paciente::novo("João", 45, "Dor no peito", true, 3),
        Paciente::novo("Maria", 32, "Fratura no braço", false, 2),
        Paciente::novo("Carlos", 60, "Apendicite", true, 3),
        Paciente::novo("Ana", 25, "Gripe forte", false, 1),
        Paciente::novo("José", 70, "Problemas cardíacos", true, 3),
        Paciente::novo("Paula", 28, "Crise de ansiedade", false, 1),
        Paciente::novo("Ricardo", 50, "Infarto leve", true, 3),
        Paciente::novo("Beatriz", 34, "Infecção urinária", false, 2),
        Paciente::novo("Fernando", 41, "Alergia grave", false, 2),
        Paciente::novo("Luiza", 29, "Corte profundo", true, 2),
        Paciente::novo("Pedro", 58, "Queda com fratura", true, 3),
        Paciente::novo("Larissa", 21, "Febre alta", false, 1),
        Paciente::novo("Tiago", 40, "Dor abdominal", false, 2),
        Paciente::novo("Sofia", 35, "Parto normal", true, 3),
        Paciente::novo("Gabriel", 47, "Pressão alta", false, 2),
        Paciente::novo("Rafaela", 19, "Crise asmática", false, 3),
        Paciente::novo("Eduardo", 65, "Cirurgia de hérnia", true, 2),
        Paciente::novo("Camila", 30, "Infecção de garganta", false, 1),
        Paciente::novo("Henrique", 55, "Cirurgia cardíaca", true, 3),
        Paciente::novo("Vanessa", 44, "Dor lombar", false, 1),
        Paciente::novo("André", 38, "Acidente de moto", true, 3),
        Paciente::novo("Juliana", 24, "Infecção intestinal", false, 2),
        Paciente::novo("Marcelo", 52, "Problemas renais", true, 2),
        Paciente::novo("Patrícia", 43, "Cefaleia intensa", false, 1),
        Paciente::novo("Felipe", 27, "Torcicolo", false, 1),
    ];

    // Ordena por prioridade 
    pacientes.sort_by(|a, b| b.prioridade.cmp(&a.prioridade));

    // Lança cada paciente em uma task independente
    let mut tarefas = vec![];
    for paciente in pacientes {
        let recursos_clone = recursos.clone();
        let tarefa = task::spawn(async move {
            paciente.atender(recursos_clone).await;
        });
        tarefas.push(tarefa);
        sleep(Duration::from_millis(150)).await;
    }

    // Aguarda o término de todos os pacientes
    for tarefa in tarefas {
        let _ = tarefa.await;
    }

    println!("\n✅ Todos os pacientes foram atendidos com sucesso!");
}
