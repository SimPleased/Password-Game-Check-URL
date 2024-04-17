use std::error::Error;
use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

async fn validate_id(video_id: String) -> Result<(bool, u8), Box<dyn Error>> {
    if video_id.len() != 11 {return Err("Video ID was not the right length.".into())}

    let periodic_elements: HashMap<&str, u8> = HashMap::from([
        ("H", 1), ("He", 2), ("Li", 3), ("Be", 4), ("B", 5), ("C", 6), ("N", 7), ("O", 8),
        ("F", 9), ("Ne", 10), ("Na", 11), ("Mg", 12), ("Al", 13), ("Si", 14), ("P", 15), ("S", 16),
        ("Cl", 17), ("Ar", 18), ("K", 19), ("Ca", 20), ("Sc", 21), ("Ti", 22), ("V", 23), ("Cr", 24),
        ("Mn", 25), ("Fe", 26), ("Co", 27), ("Ni", 28), ("Cu", 29), ("Zn", 30), ("Ga", 31), ("Ge", 32),
        ("As", 33), ("Se", 34), ("Br", 35), ("Kr", 36), ("Rb", 37), ("Sr", 38), ("Y", 39), ("Zr", 40),
        ("Nb", 41), ("Mo", 42), ("Tc", 43), ("Ru", 44), ("Rh", 45), ("Pd", 46), ("Ag", 47), ("Cd", 48),
        ("In", 49), ("Sn", 50), ("Sb", 51), ("Te", 52), ("I", 53), ("Xe", 54), ("Cs", 55), ("Ba", 56),
        ("La", 57), ("Ce", 58), ("Pr", 59), ("Nd", 60), ("Pm", 61), ("Sm", 62), ("Eu", 63), ("Gd", 64),
        ("Tb", 65), ("Dy", 66), ("Ho", 67), ("Er", 68), ("Tm", 69), ("Yb", 70), ("Lu", 71), ("Hf", 72),
        ("Ta", 73), ("W", 74), ("Re", 75), ("Os", 76), ("Ir", 77), ("Pt", 78), ("Au", 79), ("Hg", 80),
        ("Tl", 81), ("Pb", 82), ("Bi", 83), ("Po", 84), ("At", 85), ("Rn", 86), ("Fr", 87), ("Ra", 88),
        ("Ac", 89), ("Th", 90), ("Pa", 91), ("U", 92), ("Np", 93), ("Pu", 94), ("Am", 95), ("Cm", 96),
        ("Bk", 97), ("Cf", 98), ("Es", 99), ("Fm", 100), ("Md", 101), ("No", 102), ("Lr", 103), ("Rf", 104),
        ("Db", 105), ("Sg", 106), ("Bh", 107), ("Hs", 108), ("Mt", 109), ("Ds", 110), ("Rg", 111), ("Cn", 112),
        ("Nh", 113), ("Fl", 114), ("Mc", 115), ("Lv", 116), ("Ts", 117), ("Og", 118),
    ]);
    
    let roman_numerals: HashMap<char, u16>  = HashMap::from([('I', 1), ('V', 5), ('X', 10), ('L', 50), ('C', 100), ('D', 500), ('M', 1000)]);
    
    let mut xxxv: bool = false;
    let mut attomic_total: u16 = 0;
    let mut roman_numeral_values: Vec<(String, u16)> = Vec::new();
    let mut last_roman: bool = false;
    let mut next_char = None;

    for (i, c) in video_id.chars().enumerate() {
        match roman_numerals.get(&c)  {
            Some(n) => {
                if *n > 35 {
                    return Err("Roman numerals exceeded limits.".into());
                }
                if last_roman {
                    let last_index = roman_numeral_values.len() - 1;
                    let (ref mut last_numeral, ref mut last_value) = &mut roman_numeral_values[last_index];
                    *last_numeral = format!("{}{}", last_numeral, c);
                    *last_value += n;
                } else {
                    roman_numeral_values.push((c.to_string(), *n));
                }

                last_roman = true;
            }, None => {
                last_roman = false;
            }
        }

        next_char = video_id.chars().nth(i+1);
        let mut element = String::from(c);

        if next_char != None {
            element.push(next_char.unwrap());
            match periodic_elements.get(element.as_str()) {
                Some(e) => attomic_total += *e as u16,
                None => {element.pop();}
            }
        }

        match periodic_elements.get(element.as_str()) {
            Some(e) => attomic_total += *e as u16,
            None => {}
        }
    }
    
    let mut roman_numerals_total_value: u32 = 1;

    for i in 0..roman_numeral_values.len() {
        roman_numerals_total_value *= roman_numeral_values[i].1 as u32;
    }

    println!("{}", attomic_total);

    match roman_numerals_total_value {
        35 => {},
        7 => attomic_total += 23,
        5 => attomic_total += 129,
        1 => {
            if attomic_total < 49 {
                attomic_total += 152
            }
            else {
                xxxv = true;
                attomic_total += 23
            }
        }
        _ => return Err("Roman numerals exceeded limits.".into())
    }

    if attomic_total <= 200 {
        return Err("Accumulated attomic number exceeded limits.".into());
    }

    return Ok((xxxv, attomic_total as u8));
}

async fn run() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a file as an argument.");
        return;
    }

    let path = Path::new(&args[1]);
    let display = path.display();

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => panic!("Couldn't open {}: {}", display, err)
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Ok(_) => {},
        Err(err) => panic!("Couldn't read {}: {}", display, err)
    }

    let ids: Vec<&str> = s.split(',').collect();

    let mut handles = Vec::new();

    for id in ids.iter() {
        let id = id
            .replace("https://", "")
            .replace("youtube.com/watchv?=", "")
            .replace("youtu.be", "");
        let handle = tokio::spawn(async move {
            match validate_id(id.clone()).await {
                Ok(data) => println!("Video ID({}) succeeded with the attomic number ({}) with the format {}.", id, data.1, if data.0 { "XXXV" } else { "V VII" }),
                Err(err) => println!("Video ID({}) failed with the error:\n{}", id, err)
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await;
    }
}

#[tokio::main]
async fn main() {
    run().await;

    println!("Program has finished.\nPress ENTER to close.");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
