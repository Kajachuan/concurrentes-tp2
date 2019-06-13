use std::io;
use std::thread;
use std::sync::{mpsc, Arc, Barrier};
use std::collections::HashMap;
use rand::Rng;

fn get_players_number() -> u32 {
    println!("Ingrese la cantidad de jugadores:");

    let mut players_number = String::new();
    io::stdin()
        .read_line(&mut players_number)
        .expect("Error al leer la entrada");

    let mut invalid_value = true;
    let mut current_value = players_number.trim().parse::<u32>();

    if current_value.is_ok() {
        let n = current_value.clone().unwrap();
        if n >= 4 && n % 2 == 0 {
            invalid_value = false;
        } else {
            invalid_value = true;
        }
    }

    while invalid_value {
        println!("El valor ingresado es inválido: debe ser un número par mayor o igual a 4.");
        println!("Ingrese la cantidad de jugadores:");

        players_number.clear();
        io::stdin()
            .read_line(&mut players_number)
            .expect("Error al leer la entrada");

        current_value = players_number.trim().parse::<u32>();
        if current_value.is_ok() {
            let n = current_value.clone().unwrap();
            if n >= 4 && n % 2 == 0 {
                invalid_value = false;
            } else {
                invalid_value = true;
            }
        }
    }

    return current_value.unwrap();
}

fn main() {
    let players_number = get_players_number();
    println!("Coordinador: 'El juego iniciará con {} jugadores'", players_number);

    const TOTAL_CARDS: u32 = 48;

    // Baraja completa
    let mut cards = Vec::<String>::new();
    for card_value in 1..13 {
        cards.push(format!("{} de Copas", card_value));
        cards.push(format!("{} de Oros", card_value));
        cards.push(format!("{} de Bastos", card_value));
        cards.push(format!("{} de Espadas", card_value));
    }

    // Cálculo de rondas
    let rounds = TOTAL_CARDS / players_number;
    println!("Coordinador: 'Se jugarán {} rondas'", rounds);

    let actual_total_cards = rounds * players_number;

    // Barriers
    let players_barrier = Arc::new(Barrier::new(players_number as usize));
    let general_barrier = Arc::new(Barrier::new(players_number as usize + 2));

    // Canal Coordinador -> Mesa
    let (tx_coord_table, rx_coord_table) = mpsc::channel::<String>();

    // Canal Mesa -> Coordinador
    let (tx_table_coord, rx_table_coord) = mpsc::channel::<String>();

    // Canal Jugadores -> Mesa
    let (tx_players_table, rx_players_table) = mpsc::channel::<String>();

    // Canales Mesa -> JugadorX
    let mut tx_table_player = Vec::new();
    let mut rx_table_player = Vec::new();
    for _player in 0..players_number {
        let (tx, rx) = mpsc::channel::<String>();
        tx_table_player.push(tx);
        rx_table_player.push(rx);
    }

    // JUGADORES
    // Creación de jugadores
    let mut players = Vec::new();
    for player in 0..players_number {
        let rx_table_player = rx_table_player.remove(0);
        let tx_player_table = tx_players_table.clone();
        let player_barrier = players_barrier.clone();
        let player_general_barrier = general_barrier.clone();

        let new_player = thread::spawn(move || {
            let mut cards = Vec::<String>::new();
            for _card in 0..rounds {
                let card_received = rx_table_player.recv().unwrap();
                cards.push(card_received);
                println!("* El jugador {} recibió una carta, ahora tiene {} cartas *", player + 1, cards.len());
                player_barrier.wait();
            }
            player_general_barrier.wait();

            // Rondas
            for round in 0..rounds {
                let round_type = rx_table_player.recv().unwrap();
                println!("* El jugador {} escuchó que la ronda {} será {} *", player + 1, round + 1, round_type);
                let player_choice_number = rand::thread_rng().gen_range(0, 2);
                let player_choice;
                if player_choice_number == 0 {
                    player_choice = "Paso";
                }
                else {
                    player_choice = "Oxidado";
                }
                player_general_barrier.wait();
                if round_type == "hablada" {
                    println!("Jugador {}: '{}'", player + 1, player_choice);
                    tx_player_table.send(format!("{}:{}",player + 1, player_choice)).unwrap();
                    for _ in 0..players_number {
                        let other_player_choice = rx_table_player.recv().unwrap();
                        let data: Vec<&str> = other_player_choice.split(':').collect();
                        if data[0] == format!("{}", player + 1) {
                            continue;
                        }
                        println!("* El jugador {} escuchó que el jugador {} dijo {} *", player + 1, data[0], data[1]);
                    }
                }
                player_general_barrier.wait();
            }
        });

        players.push(new_player);
    }

    // MESA
    // Creación de la "mesa"
    let table_barrier = general_barrier.clone();
    let table = thread::spawn(move || {
        let mut current_player = 0;
        // Recibe y envia las cartas repartidas
        for _card in 0..actual_total_cards {
            let card_received = rx_coord_table.recv().unwrap();
            tx_table_player[current_player].send(card_received).unwrap();
            current_player = (current_player + 1) % (players_number as usize);
        }

        table_barrier.wait();

        // Rondas
        for _ in 0..rounds {
            let round_type = rx_coord_table.recv().unwrap();

            for player in 0..players_number {
                let player_round_type = round_type.clone();
                tx_table_player[player as usize].send(player_round_type).unwrap();
            }

            table_barrier.wait();
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
            table_barrier.wait();
        }

        for player in players {
            player.join().unwrap();
        }
    });

    // COORDINADOR
    // Repartición de cartas
    for _card in 0..actual_total_cards {
        let remaining_cards = cards.len();
        let card = cards.remove(rand::thread_rng().gen_range(0, remaining_cards));
        tx_coord_table.send(card).unwrap();
    }

    general_barrier.wait();

    // Rondas
    for round in 0..rounds {
        let round_type_number = rand::thread_rng().gen_range(0, 2);
        let round_type;

        if round_type_number == 0 {
            round_type = String::from("silenciosa");
        }
        else {
            round_type = String::from("hablada");
        }

        println!("Coordinador: 'Se jugará la ronda {}: esta ronda será {}'", round + 1, round_type);
        let round_type_message = round_type.clone();
        tx_coord_table.send(round_type_message).unwrap();

        if round_type_number == 1 {
            println!("Coordinador: 'Digan \"Oxidado\" o \"Paso\"'");
        }

        general_barrier.wait();

        let mut options = HashMap::<String, String>::new();
        if round_type == "hablada" {
            for _ in 0..players_number {
                let other_player_choice = rx_table_coord.recv().unwrap();
                let data: Vec<&str> = other_player_choice.split(':').collect();
                println!("* El coordinador escuchó que el jugador {} dijo {} *", data[0], data[1]);
                options.insert(String::from(data[0]), String::from(data[1]));
            }
        }
        general_barrier.wait();
    }

    table.join().unwrap();
}
