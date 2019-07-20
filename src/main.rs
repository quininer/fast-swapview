use std::fs;
use std::ffi::OsStr;
use std::path::Path;
use std::io::{ self, Read, Write };
use smallvec::SmallVec;
use rayon::prelude::*;
use bstr::{ ByteVec, ByteSlice };
use fast_floats::Fast;
use fast_lines::ReadLine;


pub fn filesize(size: isize) -> SmallVec<[u8; 8]> {
    const UNITS: [u8; 4] = [b'K', b'M', b'G', b'T'];

    fn take1(n: Fast<f64>) -> f64 {
        (n * 10.).get().round() / 10.
    }

    let mut left = Fast(size.abs() as f64);
    let mut unit = -1;

    while left > Fast(1100.) && unit < 3 {
        left = left / 1024.;
        unit += 1;
    }

    let mut output = SmallVec::new();
    if unit == -1 {
        output.extend_from_slice(itoa::Buffer::new().format(size).as_bytes());
        output.push(b'B');
    } else {
        if size < 0 {
            left = Fast(-left.get());
        }
        output.extend_from_slice(ryu::Buffer::new().format(take1(left)).as_bytes());
        output.push(UNITS[unit as usize]);
        output.extend_from_slice(b"iB");
    }
    output
}

pub fn chop_null(mut s: Vec<u8>) -> Vec<u8> {
    if let Some(0x00) = s.as_bytes().last_byte() {
        s.pop_byte();
    }

    for b in s.as_bytes_mut() {
        if *b == 0x0 {
            *b = b' ';
        }
    }

    s
}

pub fn get_comm_for(pid: &Path) -> io::Result<Vec<u8>> {
    let cmdline_path = pid.join("cmdline");
    let buf = fs::read(cmdline_path)?;
    Ok(chop_null(buf))
}

pub fn get_swap_for<R: Read>(reader: R) -> io::Result<isize> {
    fn parse(input: &[u8]) -> Option<isize> {
        use nom::bytes::complete::{ tag, take_until };
        use nom::character::complete::multispace1;

        #[inline]
        fn foo(input: &[u8]) -> Result<isize, nom::Err<()>> {
            #[cfg(feature = "rss")]
            let (input, _) = tag("Rss:")(input)?;

            #[cfg(not(feature = "rss"))]
            let (input, _) = tag("Swap:")(input)?;

            let (input, _) = multispace1(input)?;
            let (_, output) = take_until(" ")(input)?;

            let result = lexical_core::try_atoisize_slice(output);
            if let lexical_core::ErrorCode::Success = result.error.code {
                Ok(result.value)
            } else {
                Err(nom::Err::Failure(()))
            }
        }

        foo(input).ok()
    }

    let mut s = 0;
    let mut reader = ReadLine::new(reader);

    while let Some(line) = reader.read_line()? {
        if let Some(n) = parse(line) {
            s += n;
        }
    }

    Ok(s * 1024)
}

pub fn get_swap() -> io::Result<Vec<(usize, isize, Vec<u8>)>> {
    fn osstr_to_usize(input: &OsStr) -> Option<usize> {
        let b = <[u8]>::from_os_str(input)?;
        let result = lexical_core::try_atousize_slice(b);
        if let lexical_core::ErrorCode::Success = result.error.code {
            Some(result.value)
        } else {
            None
        }
    }

    // big
    let mut swapinfo = fs::read_dir("/proc")?
        .par_bridge()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let pid = path
                .file_name()
                .and_then(|name| osstr_to_usize(name))?;
            let file = fs::File::open(path.join("smaps")).ok()?;

            match get_swap_for(&file) {
                Ok(0) | Err(_) => None,
                Ok(swap) => {
                    let comm = get_comm_for(&path)
                        .ok()
                        .unwrap_or_else(Vec::new);
                    Some((pid, swap, comm))
                }
            }
        })
        .collect::<Vec<_>>();

    // FIXME better number
    if swapinfo.len() > 64 {
        swapinfo.par_sort_unstable_by_key(|&(_, size, _)| size);
    } else {
        swapinfo.sort_unstable_by_key(|&(_, size, _)| size);
    }

    Ok(swapinfo)
}

fn main() -> io::Result<()> {
    let swapinfo = get_swap()?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    writeln!(&mut stdout, "{:>5} {:>9} {}", "PID", "SWAP", "COMMAND")?;
    let mut total = 0;
    for (pid, swap, comm) in swapinfo {
        total += swap;
        writeln!(&mut stdout, "{:>5} {:>9} {}",
            itoa::Buffer::new().format(pid),
            filesize(swap).as_bstr(),
            comm.as_bstr()
        )?;
    }
    writeln!(&mut stdout, "Total: {:>8}", filesize(total).as_bstr())?;
    Ok(())
}
