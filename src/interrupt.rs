use ctrlc;

pub fn setup_interrupt_handler(){
    ctrlc::set_handler(move || {
       println!("CTRL-C received");
    }).expect("Error setting Ctrl-C handler");
}
