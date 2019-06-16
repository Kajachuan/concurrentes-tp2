use std::thread;
use std::sync::{mpsc, Arc, Barrier};

const TOTAL_CARDS: u32 = 48;

pub fn init(players:            Vec<thread::JoinHandle<()>>,
            rx_coord_table:     mpsc::Receiver<String>,
            tx_table_player:    Vec<mpsc::Sender<String>>,
            rx_players_table:   mpsc::Receiver<String>,
            tx_table_coord:     mpsc::Sender<String>,
            barrier:            Arc<Barrier>) -> thread::JoinHandle<()> {

    let players_number = players.len() as u32;
    let rounds = TOTAL_CARDS / players_number;
    let total_cards = rounds * players_number;

    return thread::spawn(move || {
        let mut current_player = 0;
        // Recibe y envia las cartas repartidas
        for _card in 0..total_cards {
            let card_received = rx_coord_table.recv().unwrap();
            tx_table_player[current_player].send(card_received).unwrap();
            current_player = (current_player + 1) % (players_number as usize);
        }

        barrier.wait();

        // Rondas
        for _ in 0..rounds {
            let round_type = rx_coord_table.recv().unwrap();

            for player in 0..players_number {
                let player_round_type = round_type.clone();
                tx_table_player[player as usize].send(player_round_type).unwrap();
            }

            barrier.wait();
            if round_type == "hablada" {
                for _ in 0..players_number {
                    let player_choice = rx_players_table.recv().unwrap();
                    let player_choice_message = player_choice.clone();
                    tx_table_coord.send(player_choice_message).unwrap();
                    for player in 0..players_number {
                        let player_choice_message = player_choice.clone();
                        tx_table_player[player as usize].send(player_choice_message).unwrap();
                    }
                }
            }
            barrier.wait();

            // Cartas
            for _ in 0..players_number {
                let player_card = rx_players_table.recv().unwrap();
                tx_table_coord.send(player_card).unwrap();
            }
            barrier.wait();

        }

        for player in players {
            player.join().unwrap();
        }
    });
}
