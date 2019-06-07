use std::io;
use std::thread;
use std::sync::mpsc;
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
    println!("Coordinador: 'El juego iniciará con {:?} jugadores'", players_number);

    const TOTAL_CARDS: u32 = 48;

    // Baraja completa
    let mut cards = Vec::new();
    for card_value in 1..13 {
        for _suit in 0..4 {
            cards.push(card_value);
        }
    }

    // Cálculo de rondas
    let rounds = TOTAL_CARDS / players_number;
    println!("Coordinador: 'Se jugarán {} rondas'", rounds);

    let actual_total_cards = rounds * players_number;

    // Canal Coordinador -> Mesa
    let (tx_coord_table, rx_coord_table) = mpsc::channel::<u32>();

    // Canal Mesa -> Coordinador
    // let (tx_table_coord, rx_table_coord) = mpsc::channel();

    // Canal Jugadores -> Mesa
    // let (tx_players_table, rx_players_table) = mpsc::channel();

    // Canales Mesa -> JugadorX
    let mut tx_table_player = Vec::new();
    let mut rx_table_player = Vec::new();
    for _player in 0..players_number {
        let (tx, rx) = mpsc::channel::<u32>();
        tx_table_player.push(tx);
        rx_table_player.push(rx);
    }

    // Creación de jugadores
    let mut players = Vec::new();
    for player in 0..players_number {
        let rx_table_player = rx_table_player.remove(0);

        let new_player = thread::spawn(move || {
            let mut cards = Vec::<u32>::new();
            for _card in 0..rounds {
                let card_received = rx_table_player.recv().unwrap();
                cards.push(card_received);
                println!("Jugador {}: Recibí una carta. Ahora tengo {} cartas", player + 1, cards.len());
            }
        });

        players.push(new_player);
    }

    // Creación de la "mesa"
    let table = thread::spawn(move || {
        let mut current_player = 0;
        for _card in 0..actual_total_cards {
            let card_received = rx_coord_table.recv().unwrap();
            tx_table_player[current_player].send(card_received).unwrap();
            current_player = (current_player + 1) % (players_number as usize);
        }

        for player in players {
            player.join().unwrap();
        }
    });

    // Repartición de cartas
    for _card in 0..actual_total_cards {
        let remaining_cards = cards.len();
        let card = cards.remove(rand::thread_rng().gen_range(0, remaining_cards));
        tx_coord_table.send(card).unwrap();
    }

    table.join().unwrap();
}
