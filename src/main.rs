
use engine_lab::tempo::Tempo;
use engine_lab::transport::Transport;
    
fn main() {
    let tempo = Tempo::new(120.0, 44_100, (4, 4));
    let mut transport = Transport::new();

    transport.play();
    transport.advance_s(44_100);

    println!("Beat position: {}", transport.beat_position(&tempo));
    println!("Bar position: {}", transport.bar_position(&tempo));
}

