pub fn vec_end_in_nl(input: &Vec<u8>) -> bool {
    let len = input.len();
    let one = input[len - 1];
    let two = input[len - 2];
    let three = input[len - 3];
    let four = input[len - 4];
    if (one != 10 && one != 13) || (two != 10 && two != 13) || (three != 10 && three != 13) || (four != 10 && four != 13) {
        return false;
    }
    true
}

// Hopefully much faster than the above one - if it doesn't end in garbage null bytes
pub fn chunk_end_in_nl(string: &String) -> bool {
    let last_four: Vec<char> = string.chars().rev().take(4).collect();
    for i in 0..4 {
        if (last_four[i] as u8) != 10 && (last_four[i] as u8) != 13 {
            return false;
        }
    }
    true
}

pub fn get_string_from_buffer_string(string: String) -> String {
	let mut out = String::new();
	for c in string.chars() {
		if (c as u8) != 0 {
			out.push(c);
		} else {
			break;
		}
	}
	out
}
