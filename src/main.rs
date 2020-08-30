use pcap_ifs as lib;

fn main() {
    let interfaces = lib::interfaces();
    interfaces.iter().for_each(|inter| {
        if inter.is_up() && !inter.is_loopback() {
            println!("{}", inter);
        }
    })
}
