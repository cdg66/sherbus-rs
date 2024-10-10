
// #![no_std]
// use vectrix::Vector;
// use core::{mem::MaybeUninit, ptr::null};

// #[warn(dead_code)]
// const ACCEPT_EVERYTHING:[u32;8] = [0;8];
// const ACCEPT_PROTOCOL:[u32;8] = [0x80000000,0,0,0,0,0,0,0];
// const ACCEPT_CONTROL:[u32;8] = [0xC0000000,0,0,0,0,0,0,0];
// const ACCEPT_INTERUPT:[u32;8] = [0xA0000000,0,0,0,0,0,0,0];
// //const ACCEPT_ECHO_ALL:[u32;8] = [0x98000000,0,0,0,0,0,0,0];
// //const ACCEPT_ECHO_:[u32;8] = [0x98000000,0,0,0,0,0,0,0];
// const ACCEPT_STREAM:[u32;8] = [0x84000000,0,0,0,0,0,0,0];
// const ACCEPT_EVERYTHING_BPI:[u32;8] = [0x81000000,0,0,0,0,0,0,0];

// const NUMBER_OF_MASK = 14;

// #[derive(Debug)]

// fn nofunc(){}

// pub struct Mask<F> 
// where 
//     F:Fn(&[u32;8]),
// {
//     masks: : [([u32; 8], F) NUMBER_OF_MASK];// = [([0; 8], nofunc); NUMBER_OF_MASK];
// }

// impl<F> Mask<F>
// where
//     F: Fn(&[u32;8]),
// {

//     pub fn new() ->Self{


//         return Mask{masks:  
//     }

//     /// add a new mask and return the mask index
//     pub fn add_mask(mut self,new_mask:[u32;8],callback: F) -> usize{
//         self.masks.push((new_mask,callback));
//         return self.masks.len()-1;// since we pushed the last index is 
//     }

//     /// remove a mask and its callback
//     pub fn remove_mask(mut self, index:usize)-> Result<usize,&str>{
//         if self.masks.is_empty(){
//             return Err("No mask to check");
//         }
//         self.masks.remove(index);
//         return Ok(index);
//     }

//     /// keep the same mask value but change the callback
//     pub fn update_callback(mut self, index:usize, callback: F)-> Result<usize,&str>{
//         if self.masks.is_empty(){
//             return Err("No mask to check");
//         }
//         // Update the value at the specified index
//         if index < self.masks.len() {
//             self.masks[index] = (self.masks[index].0,callback); // Update the value
//             return Ok(index);
//         }
//         return Err("index specified out of bound");
            
//     }

//     /// check if the mask and message are a match and if so call the associated callback
//     pub fn check(self,message:[u32; 8]) -> Result<usize,&str>{

//         let mut extracted_mesage:[u32;8] = [0;8];
//         if self.masks.is_empty(){
//             return Err("No mask to check");
//         }

        
//         for (j ,i)in self.masks.iter().enumerate(){ // check each mask
//             let mut matched = true;
//             for (index,value) in i.0.iter().enumerate(){// check each word in a message
    
//                 if (value & message[index]) != *value{ //and based masking
//                     // the message is not in the current mask 
//                     matched = false;
//                     break;
//                 }
                

//             }
//             if matched == true // we have a match perform the callback'
//             {
//                 self.masks[j].1(&extracted_mesage);
//                 return Ok(j);

//             }
//         }
//         return Err("No matched mask found");
//     }
// }

