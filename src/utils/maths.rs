// Ethan Olchik
// src/utils/compile.rs
// Cool maths stuff

pub fn jaro_distance(s: String, s2: String) -> f32 {
    if s == s2 {
        return 1.0;
    }

    let len1 = s.len();
    let len2 = s2.len();

    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let match_distance = (std::cmp::max(len1, len2) / 2) - 1;

    let mut s_matches = vec![false; len1];
    let mut t_matches = vec![false; len2];

    let mut matches = 0;

    for (i, c) in s.chars().enumerate() {
        let start = std::cmp::max(0, i as isize - match_distance as isize) as usize;
        let end = std::cmp::min(i + match_distance + 1, len2);

        for (j, c2) in s2.chars().enumerate() {
            if c == c2 && !t_matches[j] && j >= start && j < end {
                s_matches[i] = true;
                t_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut t = 0;
    let mut k = 0;

    for i in 0..len1 {
        if s_matches[i] {
            while !t_matches[k] {
                k += 1;
            }

            if s.chars().nth(i).unwrap() != s2.chars().nth(k).unwrap() {
                t += 1;
            }

            k += 1;
        }
    }

    let m = matches as f32;

    (m / len1 as f32 + m / len2 as f32 + (m - t as f32) / m) / 3.0
}

pub fn jaro_winkler(s: String, s2: String) -> f32 {
    let jaro = jaro_distance(s.clone(), s2.clone());

    let mut prefix = 0;

    for (i, c) in s.chars().enumerate() {
        if i >= s2.len() {
            break;
        }

        if c == s2.chars().nth(i).unwrap() {
            prefix += 1;
        } else {
            break;
        }
    }

    let prefix = std::cmp::min(4, prefix);

    jaro + 0.1 * prefix as f32 * (1.0 - jaro)
}