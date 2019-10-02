// -*- mode: rust; -*-
//
// This file is part of twopac.
// Copyright © 2019 Galois, Inc.
// See LICENSE for licensing information.

//! Implementation of semi-honest two-party computation.

mod evaluator;
mod garbler;

pub use evaluator::Evaluator;
pub use garbler::Garbler;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        circuit::Circuit,
        dummy::Dummy,
        util::RngExt,
        CrtBundle,
        CrtGadgets,
        Fancy,
        FancyInput,
    };
    use itertools::Itertools;
    use ocelot::ot::{ChouOrlandiReceiver, ChouOrlandiSender};
    use scuttlebutt::{AesRng, Channel};
    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixStream,
    };

    type Reader = BufReader<UnixStream>;
    type Writer = BufWriter<UnixStream>;
    type MyChannel = Channel<Reader, Writer>;

    fn addition<F: Fancy>(f: &mut F, a: &F::Item, b: &F::Item) -> Result<Option<u16>, F::Error> {
        let c = f.add(&a, &b)?;
        f.output(&c)
    }

    #[test]
    fn test_addition_circuit() {
        for a in 0..2 {
            for b in 0..2 {
                let (sender, receiver) = UnixStream::pair().unwrap();
                std::thread::spawn(move || {
                    let rng = AesRng::new();
                    let reader = BufReader::new(sender.try_clone().unwrap());
                    let writer = BufWriter::new(sender);
                    let channel = Channel::new(reader, writer);
                    let mut gb =
                        Garbler::<MyChannel, AesRng, ChouOrlandiSender>::new(channel, rng)
                            .unwrap();
                    let x = gb.encode(a, 3).unwrap();
                    let ys = gb.receive_many(&[3]).unwrap();
                    addition(&mut gb, &x, &ys[0]).unwrap();
                });
                let rng = AesRng::new();
                let reader = BufReader::new(receiver.try_clone().unwrap());
                let writer = BufWriter::new(receiver);
                let channel = Channel::new(reader, writer);
                let mut ev =
                    Evaluator::<MyChannel, AesRng, ChouOrlandiReceiver>::new(channel, rng).unwrap();
                let x = ev.receive(3).unwrap();
                let ys = ev.encode_many(&[b], &[3]).unwrap();
                let output = addition(&mut ev, &x, &ys[0]).unwrap().unwrap();
                assert_eq!((a + b) % 3, output);
            }
        }
    }

    fn relu<F: Fancy>(b: &mut F, xs: &[CrtBundle<F::Item>]) -> Option<Vec<u128>> {
        let mut outputs = Vec::new();
        for x in xs.iter() {
            let q = x.composite_modulus();
            let c = b.crt_constant_bundle(1, q).unwrap();
            let y = b.crt_mul(&x, &c).unwrap();
            let z = b.crt_relu(&y, "100%", None).unwrap();
            outputs.push(b.crt_output(&z).unwrap());
        }
        outputs.into_iter().collect()
    }

    #[test]
    fn test_relu() {
        let mut rng = rand::thread_rng();
        let n = 10;
        let ps = crate::util::primes_with_width(10);
        let q = crate::util::product(&ps);
        let input = (0..n).map(|_| rng.gen_u128() % q).collect::<Vec<u128>>();

        // Run dummy version.
        let mut dummy = Dummy::new();
        let dummy_input = input
            .iter()
            .map(|x| dummy.crt_encode(*x, q).unwrap())
            .collect_vec();
        let target = relu(&mut dummy, &dummy_input).unwrap();

        // Run 2PC version.
        let (sender, receiver) = UnixStream::pair().unwrap();
        std::thread::spawn(move || {
            let rng = AesRng::new();
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let channel = Channel::new(reader, writer);
            let mut gb =
                Garbler::<MyChannel, AesRng, ChouOrlandiSender>::new(channel, rng).unwrap();
            let xs = gb.crt_encode_many(&input, q).unwrap();
            relu(&mut gb, &xs);
        });

        let rng = AesRng::new();
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let channel = Channel::new(reader, writer);
        let mut ev =
            Evaluator::<MyChannel, AesRng, ChouOrlandiReceiver>::new(channel, rng).unwrap();
        let xs = ev.crt_receive_many(n, q).unwrap();
        let result = relu(&mut ev, &xs).unwrap();
        assert_eq!(target, result);
    }

    #[test]
    fn test_aes() {
        let circ = Circuit::parse("circuits/AES-non-expanded.txt").unwrap();

        circ.print_info().unwrap();

        let circ_ = circ.clone();
        let (sender, receiver) = UnixStream::pair().unwrap();
        let handle = std::thread::spawn(move || {
            let rng = AesRng::new();
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let channel = Channel::new(reader, writer);
            let mut gb =
                Garbler::<MyChannel, AesRng, ChouOrlandiSender>::new(channel, rng).unwrap();
            let xs = gb.encode_many(&vec![0_u16; 128], &vec![2; 128]).unwrap();
            let ys = gb.receive_many(&vec![2; 128]).unwrap();
            circ_.eval(&mut gb, &xs, &ys).unwrap();
        });
        let rng = AesRng::new();
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let channel = Channel::new(reader, writer);
        let mut ev =
            Evaluator::<MyChannel, AesRng, ChouOrlandiReceiver>::new(channel, rng).unwrap();
        let xs = ev.receive_many(&vec![2; 128]).unwrap();
        let ys = ev.encode_many(&vec![0_u16; 128], &vec![2; 128]).unwrap();
        circ.eval(&mut ev, &xs, &ys).unwrap();
        handle.join().unwrap();
    }
}
