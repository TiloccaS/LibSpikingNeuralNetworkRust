#![allow(non_snake_case)]

use std::{fmt::{Debug, Formatter}, fs::File, io::Write};
pub mod lib_neuron;

use petgraph::dot::{Dot, Config};





 mod cdl{
    use std::{sync::{Mutex, Condvar}, usize};


    #[allow(non_camel_case_types)]
    pub struct barriera{
        fix:Vec<usize>,//numero neuroni nel livello corrispondente alla cella del vettore
        val:Vec<usize>,//numero di neuroni in esecuzione per livello in ogni cella
        check:Vec<bool>,//se è true può partire il neurone i
        checkLayer:Vec<bool>,//se è true occorerà attivare il livello i
        n:usize

    }
    impl  barriera {
        pub fn new(capacity:usize,val:Vec<usize>)->barriera{
            let mut vettore = Vec::with_capacity(capacity);
            let mut vettore_check=Vec::new();
            let mut vettore_checkLayer=Vec::new();

            let mut vettore_fix=Vec::with_capacity(capacity);
            let mut numeroNeuroni=0;
            for i in 0..capacity{
                vettore_fix.push(val[i]);
                vettore.push(0);
                numeroNeuroni+=vettore_fix[i];
                vettore_checkLayer.push(false);
            }
            println!("{:?}",vettore_fix);
            for _ in 0..numeroNeuroni{
                vettore_check.push(false);
            }
            let  n=vettore_fix[0];
            let mut bar=barriera { val:vettore, fix:vettore_fix,check:vettore_check,checkLayer:vettore_checkLayer,n:n};
            bar.attivaLivello(0);
            println!("{:?}",bar.check);
            bar
        }
        pub fn attivaLivello(&mut self, layer:usize){
            let mut indicePartenza=0;
            for i in 0..layer{
                indicePartenza+=self.fix[i];

            }
            for i in 0..self.fix[layer]{
                self.check[indicePartenza+i]=true;
            }
        }
      

        
    }
    pub struct  Cdl{
        //ho bisogno di una barriera che deve essere letta in contemporanea da più thread
        m:Mutex<barriera>,
        cv:Condvar,
    }
    impl Cdl{
        pub fn new(capacity:usize,count:Vec<usize>,)->Self{
            Cdl{
                m:Mutex::new(barriera::new(capacity,count)),
                cv:Condvar::new()
            }

        }
        pub fn count_down(&self,layer:usize,_tid:usize,ultimogiro:bool){
            let mut s=self.m.lock().unwrap();
            let  layer=layer-1;
            s.n-=1;
            s.val[layer]+=1;
        
            if s.val[layer]==s.fix[layer]{
                s.checkLayer[layer]=false;
                s.val[layer]=0;
                match (layer,ultimogiro){
                    (1,true) => {s.checkLayer[layer+1]=true;},
                    (1,false) => {
                        s.checkLayer[layer-1]=true;
                        s.checkLayer[layer+1]=true;
                    },
                    (n,_) if n !=1 && n<s.val.len()-1 =>{ s.checkLayer[layer+1]=true;}
                    (_,_) => {}
                }
            }
            if s.n==0{
                for i in 0..s.checkLayer.len(){
                    if s.checkLayer[i]==true{
                        println!("ho attivato il livello:{:?}",i);
                        s.attivaLivello(i);
                        s.n+=s.fix[i];
                    }
                }
            }
            self.cv.notify_all();
        }
        
        pub fn wait(&self,layer:usize,tid:usize){
            let mut s=self.m.lock().unwrap();
            let  _layer=layer-1;  
                while s.check[tid]==false {//finche sono false dormos
                    s=self.cv.wait(s).unwrap();
                }
        
            s.check[tid]=false;
        }
    }
}

pub mod NeuralNetwork{
    
    use crate::lib_neuron::Neuron;
    use std::fmt::{ Debug, Formatter};  
    use std::sync::{Arc,Mutex};
    use  petgraph::{Graph,Direction};
    use petgraph::adj::NodeIndex;
    use std::time::{Instant};
    use std::thread::{ JoinHandle};
    use std::thread;
    use rand::Rng;
    use std::fs::File;
    use std::io::{BufRead,BufReader};
    use super::cdl;


#[derive(Clone)]
pub struct NeuralNetwork  {
  pub  Network:Graph<Neuron,f64>,
   pub out:Vec<Impulse>,
   pub levels:usize
}

#[derive(Clone)]
pub struct Impulse{
   pub  val:usize,
   pub tempo:Instant

}

impl Debug for Impulse  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"\nimpulso ricevuto nell'istante: {:?}",self.tempo)
    }
    
}
impl Impulse{
    pub fn new(val:usize)->Impulse{
        Impulse{
            val,
            tempo:Instant::now()
        }
    }
}
impl  NeuralNetwork  {
    pub fn new(Neurons:Vec<Vec<Neuron>>,WeigthSameLevel:Vec<f64>,WeigthNextLevel:Vec<f64>)->NeuralNetwork{
        let mut GRAFO = Graph::<Neuron, f64>::new();
        Neurons.iter().flatten().for_each(|n| {GRAFO.add_node(n.clone() );});
        let mut vec_same_level= Vec::<>::new();
        let mut vec_next_level= Vec::<>::new();
        let mut cnt_same_level=0;
        let mut cnt_next_level=0;
        for neurone1 in GRAFO.node_indices() {
            let n1 = GRAFO.node_weight(neurone1).unwrap().clone();
            for neurone2 in GRAFO.node_indices().filter( |n| {return if GRAFO.node_weight(*n).unwrap().Nid == n1.Nid {false} else {true}  } ){
                let n2  =GRAFO.node_weight(neurone2).unwrap();
                if n1.layer == n2.layer {
                    vec_same_level.push(( neurone1,neurone2,WeigthSameLevel[cnt_same_level]) );
                    cnt_same_level+=1;
                }
                if n2.layer == n1.layer+1 {
                    vec_next_level.push(( neurone1,neurone2,WeigthNextLevel[cnt_next_level]));
                    cnt_next_level+=1;
                }
            }
        }
        vec_same_level.iter().for_each(|(n1,n2,w)| {GRAFO.add_edge(*n1, *n2, *w);});
        vec_next_level.iter().for_each(|(n1,n2,w)| {GRAFO.add_edge(*n1, *n2, *w);});

        NeuralNetwork {
            Network: GRAFO,
            out:Vec::new(),
            levels:Neurons.len(),
        }

    }
    pub fn new_from_arr(arr:&[u8],WeigthSameLevel:Vec<f64>,WeigthNextLevel:Vec<f64>)->NeuralNetwork{
        let mut rng = rand::thread_rng();
        let mut nid_counter=0;
        let mut Neurons = Vec::new();
        for (numero_livello,n) in arr.iter().enumerate(){
            let mut vec_n_liv=Vec::new();
            for _ in 0..*n{
                vec_n_liv.push(Neuron::new(nid_counter,rng.gen_range(0. .. 1.),rng.gen_range(0. .. 1.),rng.gen_range(1. .. 2.),numero_livello+1 as usize));
                nid_counter+=1
            }
            Neurons.push(vec_n_liv);
        }

        NeuralNetwork::new(Neurons, WeigthSameLevel, WeigthNextLevel)
    }
    pub fn new_from_file(filepath:String)->NeuralNetwork{

        let file = File::open(filepath).expect("file not found!"); //file of the pretrained model

        let mut lines=BufReader::new(file).lines(); //read line by line

        let mut cnt=0;
        let mut GRAFO = Graph::<Neuron, f64>::new();

        let nodes_string=lines.next().expect(" ").unwrap(); //vector of dimensions es.: [4,3,2,4,5,4,6]
        let mut nodes_per_layer: Vec<u32> = Vec::new();
        let pat: &[_] = &[' ','[',']'];
        nodes_string.trim_matches(pat).split(", ").collect::<Vec<&str>>().iter().for_each(|p|{nodes_per_layer.push(p.parse::<u32>().unwrap());});

        let mut neuron_par=Vec::new();  //parameters of the neuron potentials: reset, rest and threshold

        for (_n,_l) in nodes_per_layer.iter().enumerate(){
            for _j in 0..*_l{
                let neuron_string=lines.next().expect(" ").unwrap();
                neuron_par.clear();
                neuron_string.trim_matches(pat).split(",")
                    .collect::<Vec<&str>>().iter().for_each(|p| {neuron_par.push(p.trim().parse::<f64>().unwrap());});

                let neuron=Neuron::new(cnt,neuron_par[0],neuron_par[1],neuron_par[2],_n+1);
                GRAFO.add_node(neuron );
                cnt+=1;
            }
        }

        let mut edge_weights= Vec::<>::new();

        for idx1 in GRAFO.node_indices() {
            let n1 = GRAFO.node_weight(idx1).unwrap().clone();
            for idx2 in GRAFO.node_indices().filter( |n| {return if GRAFO.node_weight(*n).unwrap().Nid == n1.Nid {false} else {true}  } ){
                let n2  =GRAFO.node_weight(idx2).unwrap();
                if n1.layer == n2.layer || n2.layer == n1.layer+1 {
                    let edgeweight = lines.next().expect(" ").unwrap().parse::<f64>().unwrap();
                    edge_weights.push((idx1,idx2,edgeweight) );
                }
            }
        }
        edge_weights.iter().for_each(|(n1,n2, w)| {GRAFO.add_edge(*n1, *n2, *w);});

        NeuralNetwork {
            Network: GRAFO,
            out:Vec::new(),
            levels:nodes_per_layer.len(),
        }
    }

    fn find_neighbors(&mut self,nid:u32)->Vec< Neuron>{
        let mut vettoreNodi=Vec::new();
        for i in self.Network.neighbors_directed(NodeIndex::from(nid),Direction::Outgoing){

            let y=self.Network.node_weight(i).unwrap().clone();
            if vettoreNodi.contains(&y){
                    continue;
            }
            else{

                vettoreNodi.push(self.Network[i]);
            }
        }
        vettoreNodi

    }
    fn get_layer(&mut self,layer:usize)->Vec<NodeIndex>{
        let mut vett=Vec::new();
        for &i in self.Network.node_weights(){
                if i.layer==layer  {
                    vett.push(i.Nid.clone());
                }
        }
        vett
    }
    fn edit_neuron(&mut self,neuroni:Vec<Neuron>){
        for i in neuroni{
            //println!("aggiornamenti di {:?} che aveva impulsi: {:?}",i.Nid,self.Network.node_weight(NodeIndex::from(i.Nid)).unwrap().get_impulse_neuron());
            self.Network.node_weight_mut(NodeIndex::from(i.clone().Nid)).unwrap().aggiorna_neurone(i);

        }
    }


    pub fn start_simulation(mut self,vettoreImpulsi:Vec<usize>)->Vec<Impulse>{

        let vettore_fix=(0..self.levels).map(|i| self.get_layer(i+1).len() ).collect::<Vec<usize>>();//vettore con numero di nodi per cella
        let cdl=Arc::new(cdl::Cdl::new(vettore_fix.len(),vettore_fix));
        let rete=Arc::new(Mutex::new(self.clone()));
        let neuroni=self.get_neuroni();

        let h:Vec<JoinHandle<()>>=neuroni.into_iter().map(|mut tid|
        {
            let  vettoreVicini=self.clone().find_neighbors(tid.Nid).clone();
            let  rete=rete.clone();
            let  cdl=Arc::clone(&cdl);
            let  mut rete3=self.clone();
            let  impulsiIngresso=vettoreImpulsi.clone();
            let len_layer1=rete3.get_layer(1).len();
            let  n=impulsiIngresso.clone().len()/len_layer1;
            thread::spawn(move||{
                for i in 1..=n
                {
                cdl.wait(tid.layer,tid.Nid as usize);
                tid=rete.lock().unwrap().Network.node_weight(NodeIndex::from(tid.Nid)).unwrap().clone();
                rete.lock().unwrap().Network.node_weight_mut(NodeIndex::from(tid.Nid)).unwrap().delete_impulse_in_neurone();
                println!("tid: {:?} livello;{:?}",tid.Nid,tid.layer-1);
                if tid.check_neurone(impulsiIngresso.clone(),(i-1)*len_layer1)==true{
                    //println!("{:?} sta mandando impulsi",tid.Nid);
                    let vett=tid.send_impulse( vettoreVicini.clone(), rete3.clone());
                    if tid.layer==rete3.levels{
                        rete.lock().unwrap().out.push(Impulse::new(1));
                    }
                    rete.lock().unwrap().edit_neuron(vett);
                    tid.potential_membrane=tid.potential_reset;
                    rete.lock().unwrap().Network.node_weight_mut(NodeIndex::from(tid.Nid)).unwrap().editNeurone(tid.clone());

                }else{
                    //println!("{:?} non ha raggiunto la soglia",tid.Nid);
                    rete.lock().unwrap().Network.node_weight_mut(NodeIndex::from(tid.Nid)).unwrap().editNeurone(tid.clone());

                }
                if i ==n {
                    cdl.count_down(tid.layer,tid.Nid as usize,true);

                }
                else{
                    cdl.count_down(tid.layer,tid.Nid as usize,false);
                }

                }
            })
        }).collect();
        h.into_iter().for_each(|ti|{ti.join().unwrap()});
        let vetOut=rete.lock().unwrap().out.clone();
        vetOut
    }
fn get_neuroni(&mut self)->Vec<Neuron>{
    self.Network.node_weights().map(|n| *n).collect()
}
   }

   #[cfg(test)]
   mod unit_tests {
    use super::*;
    use petgraph::dot::{Dot, Config};
    #[test]
    fn test_create() {
        let rete=NeuralNetwork::new_from_arr(&[2,2], Vec::from([-1.0, -1.0,-1.,-1.]), Vec::from([2.0, 2.0,2.0,2.0]));
        let output = format!("{}", Dot::with_config(&rete.Network,&[Config::NodeNoLabel]));
        assert_eq!(output, "digraph {\n    0 [ ]\n    1 [ ]\n    2 [ ]\n    3 [ ]\n    0 -> 1 [ label = \"-1\" ]\n    1 -> 0 [ label = \"-1\" ]\n    2 -> 3 [ label = \"-1\" ]\n    3 -> 2 [ label = \"-1\" ]\n    0 -> 2 [ label = \"2\" ]\n    0 -> 3 [ label = \"2\" ]\n    1 -> 2 [ label = \"2\" ]\n    1 -> 3 [ label = \"2\" ]\n}\n");
    }
    #[test]
    fn test_get_neuroni(){
        let mut rete=NeuralNetwork::new_from_arr(&[2], Vec::from([-1.0, -1.0,-1.0,-1.0]), Vec::from([2.0, 2.0,2.0,2.0]));
        assert_eq!(rete.get_neuroni().into_iter().map(|n| {n.Nid}).collect::<Vec<u32>>(),[0,1]);
    }

    }
}



impl Debug for NeuralNetwork::NeuralNetwork  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        let mut dotFile = File::create("network.dot").unwrap();
        let output = format!("{}", Dot::with_config(&self.Network,&[Config::NodeNoLabel,Config::EdgeNoLabel]));  //, &[Config::EdgeNoLabel]        //save the dot representation of the network to file
        //dotFile.write_all(&output.as_bytes()).expect("could not write file");
      dotFile.write_all(&output.as_bytes()).expect("could not write file");
        use std::process::Command;
        //execute command: fdp -Tsvg example.dot -o outfile.svg
        Command::new("fdp").arg("-Tsvg").arg("network.dot").arg("-o").arg("network.svg")
            .spawn()
            .expect("ls command failed to start");

        //let mut textRepr="";

        write!(f,"dot representation: {}",output)
    }
    
}
