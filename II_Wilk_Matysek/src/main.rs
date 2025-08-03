use std::{
    env,
    ops::{BitAnd, BitOr, Not},
    str::FromStr,
};

/// 128 bitowy adres IPv6, przechowywany jako dwa u64.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct IPv6Addr {
    high: u64,
    low: u64,
}

impl BitAnd for IPv6Addr {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self { high: self.high & other.high, low: self.low & other.low }
    }
}

impl BitOr for IPv6Addr {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self { high: self.high | other.high, low: self.low | other.low }
    }
}

impl Not for IPv6Addr {
    type Output = Self;
    fn not(self) -> Self {
        Self { high: !self.high, low: !self.low }
    }
}

/// Prefiks IPv6 w postaci adres + długość maski.
#[derive(Debug)]
struct IPv6Prefix {
    addr: IPv6Addr,
    len: u8,
}

impl FromStr for IPv6Prefix {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ip_str, len_str) =
            s.split_once('/').ok_or("Brak ‘/’ w prefiksie".to_string())?;
        let len: u8 = len_str.parse()
            .map_err(|_| "Niepoprawna długosc maski".to_string())?;
        if len > 128 { return Err("Maska > 128".into()); }

        let parts: Vec<&str> = ip_str.split("::").collect();
        if parts.len() > 2 { return Err("Za duzo ‘::’".into()); }

        let head = if parts[0].is_empty() { vec![] } else { parts[0].split(':').collect() };
        let tail = if parts.len()==2 && !parts[1].is_empty() {
            parts[1].split(':').collect()
        } else { vec![] };

        if head.len() + tail.len() > 8 {
            return Err("Zbyt wiele segmentow IPv6".into());
        }

        let mut segs = Vec::with_capacity(8);
        for h in &head {
            segs.push(u16::from_str_radix(h, 16)
                .map_err(|_| "Bledny segment IPv6".to_string())?);
        }
        for _ in 0..(8 - head.len() - tail.len()) { segs.push(0); }
        for t in &tail {
            segs.push(u16::from_str_radix(t, 16)
                .map_err(|_| "Bledny segment IPv6".to_string())?);
        }

        let high = ((segs[0] as u64) << 48)
            | ((segs[1] as u64) << 32)
            | ((segs[2] as u64) << 16)
            | (segs[3] as u64);
        let low  = ((segs[4] as u64) << 48)
            | ((segs[5] as u64) << 32)
            | ((segs[6] as u64) << 16)
            | (segs[7] as u64);

        Ok(IPv6Prefix { addr: IPv6Addr { high, low }, len })
    }
}

impl IPv6Prefix {
    /// Maska bitowa jako IPv6Addr
    fn mask(&self) -> IPv6Addr {
        let m = self.len;
        // unikamy paniki
        if m == 0 {
            return IPv6Addr { high: 0, low: 0 };
        }
        let high = if m >= 64 { u64::MAX } else { (!0u64) << (64 - m) };
        let low  = if m <= 64 { 0 } else { (!0u64) << (128 - m) };
        IPv6Addr { high, low }
    }

    /// Zwraca (pierwszy_adres, ostatni_adres) w prefiksie
    fn range(&self) -> (IPv6Addr, IPv6Addr) {
        let m = self.mask();
        let net = self.addr & m;
        let bcast = net | !m;
        (net, bcast)
    }

    /// Czy dwa prefiksy mają wspólny fragment?
    fn overlaps(&self, other: &Self) -> bool {
        let (s1, e1) = self.range();
        let (s2, e2) = other.range();
        s1 <= e2 && s2 <= e1
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Użycie: {} <prefiks1> <prefiks2>", args[0]);
        std::process::exit(1);
    }
    let p1: IPv6Prefix = args[1].parse().expect("Bledny pierwszy prefiks");
    let p2: IPv6Prefix = args[2].parse().expect("Bledny drugi prefiks");

    println!("{}", if p1.overlaps(&p2) { "tak" } else { "nie" });
}
