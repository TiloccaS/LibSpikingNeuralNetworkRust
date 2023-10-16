#![allow(non_snake_case)]
use std::thread;
use std::thread::JoinHandle;
use std::{
    f64::consts::E,
    fmt::{Display, Formatter},
    time::Instant,
};

use petgraph::adj::NodeIndex;

use crate::NeuralNetwork;



#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Neuron {
    pub Nid: u32,
    pub potential_reset: f64,
    pub potential_sleep: f64,
    pub threshold: f64,
    pub potential_membrane: f64,
    pub t_old: Instant,
    pub layer: usize,
    pub impulse_in: f64,
}

impl Neuron {
    pub fn new<T>(
        Nid: u32,
        potential_reset: T,
        potential_sleep: T,
        threshold: T,
        layer: usize,
    ) -> Neuron
    where
        T: std::convert::Into<f64>,
    {
        let pr = potential_reset.into();
        let  ps = potential_sleep.into();
        let  t = threshold.into();

        Neuron {
            Nid,
            potential_reset: pr,
            potential_sleep: ps,
            threshold: t,
            potential_membrane: ps,
            t_old: Instant::now(),
            layer,
            impulse_in: 0.,
        }
    }
    pub fn editNeurone(&mut self, neurone: Neuron) {
        self.potential_membrane = neurone.potential_membrane;
    }
    pub fn aggiorna_neurone(&mut self, neurone: Neuron) {
        self.impulse_in += neurone.impulse_in;
    }
    pub fn send_impulse(
        &mut self,
        vettoreNodi: Vec<Neuron>,
        rete:NeuralNetwork::NeuralNetwork
    ) -> Vec<Neuron> {
        let mut vett = Vec::new();
        let h: Vec<JoinHandle<Neuron>> = vettoreNodi
            .clone()
            .into_iter()
            .map(|mut tid| {
                let nid=self.Nid.clone();
                let  rete=rete.clone();
                thread::spawn(move || {
                    //println!(" {:?} sta mandando gli impulsi a {:?}",sx,tid.Nid);
                    //println!(" {:?} ha potenziale di mebrana {:?}",tid.Nid,tid.potential_membrane);

                    let index=rete.Network.find_edge(NodeIndex::from(nid), NodeIndex::from(tid.Nid));
                    tid.impulse_in=rete.Network.edge_weight(index.unwrap()).unwrap().clone();

                    tid
                    //println!("{:?} ha calcolato l'impulso da {:?}",tid.Nid,sx);
                    //println!(" ora {:?} ha potenziale di mebrana {:?}",tid.Nid,tid.potential_membrane);
                })
            })
            .collect();

        h.into_iter().for_each(|tid| {
            vett.push(tid.join().unwrap());
        });
        vett
    }
    pub fn get_impulse_neuron(self) -> f64 {
        self.impulse_in
    }
    pub fn delete_impulse_in_neurone(&mut self){
        self.impulse_in=0.;

    }
   
    pub fn check_neurone(&mut self,impulsiIngresso:Vec<usize>,i:usize) -> bool {
        let tau = 1;
        if self.layer==1{
          
            self.impulse_in=impulsiIngresso.get((self.Nid)as usize+i ).unwrap().clone() as f64;
        }
        
        let t_s = Instant::now();
        let elapsed = (t_s.duration_since(self.t_old).as_millis() / tau) as f64 / 1000.0;
        let decay = (self.potential_membrane - self.potential_sleep) * E.powf(-elapsed);
        self.potential_membrane = self.potential_sleep + decay + self.impulse_in;
        //println!("{:?} ha potenziale di membrana {:?} e threshold {:?} e aveva impulsi in ingresso {:?}",self.Nid,self.potential_membrane,self.threshold, self.impulse_in);
        self.impulse_in = 0.;
       
        if self.potential_membrane > self.threshold {
            self.potential_membrane=self.potential_reset;
            true
        } else {

            false
        }
    }
}
impl Display for Neuron {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.Nid)
    }
}
