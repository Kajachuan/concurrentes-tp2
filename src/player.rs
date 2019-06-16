use std::thread;
use std::sync::{mpsc, Arc, Barrier};
use rand::Rng;

const TOTAL_CARDS: u32 = 48;

pub fn init(player_number:      u32,
            players_number:     u32,
            rx_table_player:    mpsc::Receiver<String>,
            tx_player_table:    mpsc::Sender<String>,
            barrier:            Arc<Barrier>) -> thread::JoinHandle<()> {

    let rounds = TOTAL_CARDS / players_number;

    return thread::spawn(move || {
        let mut cards = Vec::<String>::new();
        for _card in 0..rounds {
            let card_received = rx_table_player.recv().unwrap();
            cards.push(card_received);
            println!("* El jugador {} recibió una carta, ahora tiene {} cartas *", player_number, cards.len());
        }
        barrier.wait();

        // Rondas
        for round in 0..rounds {
            let round_type = rx_table_player.recv().unwrap();
            println!("* El jugador {} escuchó que la ronda {} será {} *", player_number, round + 1, round_type);
            let player_choice_number = rand::thread_rng().gen_range(0, 2);
            let player_choice;
            if player_choice_number == 0 {
                player_choice = "Paso";
            }
            else {
                player_choice = "Oxidado";
            }
            barrier.wait();
            if round_type == "hablada" {
                println!("Jugador {}: '{}'", player_number, player_choice);
                tx_player_table.send(format!("{}:{}", player_number, player_choice)).unwrap();
                for _ in 0..players_number {
                    let other_player_choice = rx_table_player.recv().unwrap();
                    let data: Vec<&str> = other_player_choice.split(':').collect();
                    if data[0] == format!("{}", player_number) {
                        continue;
                    }
                    println!("* El jugador {} escuchó que el jugador {} dijo {} *", player_number, data[0], data[1]);
                }
            }
            barrier.wait();

            // Cartas
            let current_card = cards.pop().unwrap();
            tx_player_table.send(format!("{}:{}", player_number, current_card)).unwrap();
            barrier.wait();
        }
    });
}
