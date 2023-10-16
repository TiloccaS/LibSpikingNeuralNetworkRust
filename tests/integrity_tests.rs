#![allow(non_snake_case)]


#[cfg(test)]
mod testsNeuron {
    use libSpikingNeuralNetwork::lib_neuron::Neuron;

    #[test]
    fn create_neuron() {
        let n = Neuron::new(0,0,1,1,0);
        assert_eq!(n.Nid,0);
        assert_eq!(n.potential_reset,0.);
        assert_eq!(n.potential_sleep,1.);
        assert_eq!(n.threshold,1.);
        assert_eq!(n.layer,0);
    }
    #[test]
    fn edit_neuron() {
        let mut n = Neuron::new(0,0,1,1,0);
        let mut n2 = Neuron::new(0,0,1,1,0);
        n2.impulse_in=1.;
        n2.potential_membrane=3.;
        n.editNeurone(n2);
        assert_eq!(n.Nid,0);
        assert_eq!(n.potential_reset,0.);
        assert_eq!(n.potential_sleep,1.);
        assert_eq!(n.threshold,1.);
        assert_eq!(n.layer,0);
        assert_eq!(n.potential_membrane,3.);
        assert_eq!(n.impulse_in,0.);
    }
}
#[cfg(test)]
mod testsNeuralNetwork{
    use rand::Rng;
    use std::fs::File; //to write the init file
    use std::io::Write;
    use libSpikingNeuralNetwork::lib_neuron::Neuron;
    use libSpikingNeuralNetwork::NeuralNetwork::NeuralNetwork;
    #[test]
    pub fn newNeuralNetwork(){
      let mut a=Vec::new();
      let mut b =Vec::new();
      let mut cnt=0;
      let mut weight_same_level=Vec::new();
      let mut weight_next_level=Vec::new();
      for _n in 1..4{
          for _j in 1..4{
              let mut rng = rand::thread_rng();
              let neurone_temp=Neuron::new(cnt,rng.gen_range(0 .. 1),rng.gen_range(0 .. 3),rng.gen_range(0 .. 1),_n );
              for _k in 0..2{ weight_same_level.push(rng.gen_range(-4.0 .. 0.0)); }
              for _k in 0..3{weight_next_level.push(rng.gen_range(0.0 .. 4.0));   }
              cnt+=1;
              b.push(neurone_temp);
          }
          a.push(b.clone());
          b.clear();
      }
        let rete=NeuralNetwork::new(a.clone(), weight_same_level, weight_next_level);
        assert_eq!(rete.Network.node_count(),9);
        assert_eq!(rete.Network.edge_count(),9*5-3*3)
    }

    #[test]
    pub fn NeuralNetworkSendImpulsseCorrect(){
        let mut a=Vec::new();
        let mut b =Vec::new();
        let mut cnt=0;
        let mut weight_same_level=Vec::new();
        let mut weight_next_level=Vec::new();
        for _n in 1..6{
              for _j in 1..5{
                  let mut rng = rand::thread_rng();
                  let neurone_temp=Neuron::new(cnt,rng.gen_range(0. .. 1.),rng.gen_range(0. .. 1.),rng.gen_range(1. .. 2.),_n );
                  for _k in 0..3{ weight_same_level.push(rng.gen_range(-4.0 .. 0.0)); }
                  for _k in 0..4{ weight_next_level.push(rng.gen_range(0.0 .. 4.0));  }
                  cnt+=1;
                  b.push(neurone_temp);
              }
     
              a.push(b.clone());
              b.clear();
          }
          let rete=NeuralNetwork::new(a.clone(), weight_same_level, weight_next_level);
          let mut vett=Vec::new();
          for _ in 1..=4*5{
            let  _rng = rand::thread_rng();
            vett.push(1);
            
          }
         let  ImpulseOut= rete.clone().start_simulation(vett);
          println!("{:?}",rete);
          println!("{:?}",ImpulseOut);

    }

    #[test]
    fn NeuralNetworkDeafaultArrSendImpulse(){
        let mut weight_same_level=Vec::new();
        let mut weight_next_level=Vec::new();
        for _ in 0..60{weight_same_level.push(-1.)}
        for _ in 0..30{weight_next_level.push(2.)}
        let rete = NeuralNetwork::new_from_arr(&[5,1,1,5,1,1,5,1,1,1,1], weight_same_level,weight_next_level);
        let mut vett=Vec::new();
        for _ in 1..=rete.Network.node_count(){
            let  _rng = rand::thread_rng();
            vett.push(1);

        }
        let  ImpulseOut= rete.clone().start_simulation(vett);
        println!("{:?}",rete);
        println!("{:?}",ImpulseOut);
    }

    #[test]
    pub fn new_NN_from_file() {  //write a random initialization file and test new_from_file
        println!("test: new Neural Network from file");
        let nodes_per_layer: Vec<u32> = Vec::from([4, 3, 12, 4, 5, 4, 6]); //network dimensions
        let mut file_init = File::create("model.txt").expect("file not found!");

        file_init.write(format!("{:?}\n", nodes_per_layer).as_bytes()).expect("Unable to write file");

        let mut rng = rand::thread_rng();

        //write random neuron parameters
        for n in nodes_per_layer.iter() {
            for _ in 0..*n {
                let mut neuron_param = Vec::new();
                for _i in 0..3 {
                    neuron_param.push(rng.gen_range(0. .. 1.));
                }
                file_init.write(format!("{:?}\n", neuron_param).as_bytes()).expect("Unable to write file");
            }
        }
        //write random edge weights
        for (index, n) in nodes_per_layer.iter().enumerate() {
            for i in 0..*n {
                for j in 0..*n { //edges of the same layer
                    if i != j {
                        let edgeweight = rng.gen_range(-1. .. 0.);
                        file_init.write(format!("{:?}\n", edgeweight).as_bytes()).expect("Unable to write file");
                    }
                }
                if index < nodes_per_layer.len() - 1 { //except last layer
                    for _ in 0..*nodes_per_layer.get(index + 1).unwrap() { //edges to the next layer
                        let edgeweight = rng.gen_range(0. .. 1.);
                        file_init.write(format!("{:?}\n", edgeweight).as_bytes()).expect("Unable to write file");
                    }
                }
            }
        }
        let mut impulse_in = Vec::new();
        for _ in 1..=4 * 7 {
            impulse_in.push(1);
        }
        let rete = NeuralNetwork::new_from_file(String::from("./model.txt"));
        assert_eq!(rete.Network.node_count(),38);
        assert_eq!(rete.Network.edge_count(),384);

        let ImpulseOut = rete.clone().start_simulation(impulse_in);
        println!("{:?}", rete);
        println!("output: {:?}", ImpulseOut);
    }

}