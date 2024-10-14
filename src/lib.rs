#![no_std]

//use cortex_m;
//use embedded_hal::delay::DelayNs;

use parser::parser::{parse_rx,parse_tx,PacketParsed,RawPacket,PacketType};
use rp2040_hal::{
    gpio::{AnyPin, Function, FunctionPio0, FunctionPio1, Pin, PinId},
    pio::{Buffers, PIOBuilder, PIOExt, PinDir, Running, Rx, ShiftDirection, StateMachine, StateMachineIndex, Tx, UninitStateMachine, ValidStateMachine, PIO},
};
use core::prelude::v1::Option;



//use mask::Mask;

// Raw implementation of the driver
pub struct SherBusRaw<'a, P, SMTX,SMRX,SMC, TX,TXEN,RX,C/* ,F*/>
where
    TX: AnyPin<Function = P::PinFunction>,
    TXEN: AnyPin<Function = P::PinFunction>,
    RX: AnyPin<Function = P::PinFunction>,
    C: AnyPin<Function = P::PinFunction>,
    P: PIOExt,
    SMTX: StateMachineIndex,
    SMRX: StateMachineIndex,
    SMC: StateMachineIndex,
    /*F: Fn(&[u32;8]),*/


{

    tx: Tx<(P, SMTX)>,
    rx: Rx<(P,SMRX)>,
    txsm: StateMachine<(P,SMTX),Running>,
    rxsm: StateMachine<(P,SMRX),Running>,
    csm: StateMachine<(P,SMC),Running>,
    _tx_pin: TX,
    _tx_en_pin: TXEN,
    _rx_pin: RX,
    _coll_pin: C,
    pio: &'a mut PIO<P>,
    //pub masks: Mask<F>,
}
impl <'a,P, SMTX,SMRX,SMC, TX,TXEN,RX,C/*,F*/> SherBusRaw<'a,P, SMTX,SMRX,SMC, TX,TXEN,RX,C/* ,F*/> 
where 
    TX: AnyPin<Function = P::PinFunction>,
    TXEN: AnyPin<Function = P::PinFunction>,
    RX: AnyPin<Function = P::PinFunction>,
    C: AnyPin<Function = P::PinFunction>,
    P: PIOExt,
    SMTX: StateMachineIndex,
    SMRX: StateMachineIndex,
    SMC: StateMachineIndex,
    //F: Fn(&[u32;8]),
{

    pub fn new(
        d: TX,
        den:TXEN,
        r: RX,
        c: C,
        pio: &'a mut PIO<P>,
        smtx: UninitStateMachine<(P, SMTX)>,
        smrx: UninitStateMachine<(P, SMRX)>,
        smc: UninitStateMachine<(P, SMC)>,
        //clock_freq: fugit::HertzU32,

    )-> Self{
        // Tx pio machine
        let program_tx = pio_proc::pio_asm!(
            ".side_set 1", // each instruction may set 1 bit
            ".wrap_target",
            "wait 1 irq 0 side 0", // wait before the cpu tell the pio to go
            "irq set 1 side 0", // raise a flag saying that the sm is currently sending data
            //"pull block side 0",
            "set pins 1 side 1",
            "nop side 1",
            "set pins 0 side 1",
            "nop side 1",
            //"set x 31 side 1",
            "send_256:",
            "out pins 1 side 1",
            "jmp !osre send_256 side 1",
            "irq clear 1 side 0", // tell to the cpu that the sm is free
            //"irq set 2 side 0", // set irq to tell the program that the transmit is finished and wait an acknoledge
            //"irq wait 0 side 0",
            ".wrap"


            options(max_program_size = 32)
        );

        let program_rx = pio_proc::pio_asm!(
            ".wrap_target",
            "wait 1 pin 0",
            "wait 0 pin 0",
            "nop",
            "read_until_full:",
            "in pins 1",
            "jmp read_until_full", //
            /*"wait 0 pin 0",
            "read_256:",
            "set x 31",
            "read_32:",
            "in pins 1",
            "jmp x-- read_32",
            "jmp y-- read_256",
            "irq set 2", //set a flag*/
            ".wrap"


            options(max_program_size = 32)
        );

        let collision = pio_proc::pio_asm!(
            ".wrap_target",
            "wait 1 irq 0 ",// wait that we are writing
            "nop",
            "nop", //

            "nocoll:"
            "jmp pin coll", //while there is no coll continue
            "jmp nocoll",
            "coll:"         // tell the cpu a wait to restart
            "irq wait 2"
            ".wrap"
            options(max_program_size = 32)
        );

        let d = d.into();
        let den = den.into();
        let r = r.into();
        let c = c.into();

        // Initialize and start tx machine
        let installed = pio.install(&program_tx.program).unwrap();
        let (int, frac) = (1, 0); //TODO: pass a freq and get values
        let (mut tx0, _, mut tx) = PIOBuilder::from_installed_program(installed)
            .set_pins(d.id().num, 1)
            .side_set_pin_base(den.id().num)
            .autopull(true)
            //.autopush(true)
            .buffers(Buffers::OnlyTx)
            .clock_divisor_fixed_point(int, frac)
            .pull_threshold(32)
            .out_pins(d.id().num, 1)
            .out_shift_direction(ShiftDirection::Left)
            .build(smtx);
        // The GPIO pin needs to be configured as an output.
        //sm.set_pindirs([(led_pin_id, hal::pio::PinDir::Output)]);
        tx0.set_pindirs([(d.id().num, PinDir::Output)]);
        tx0.set_pindirs([(den.id().num, PinDir::Output)]);

        // Initialize and start rx machine
        let installed2 = pio.install(&program_rx.program).unwrap();
        let (mut rx0, mut rx, _) = PIOBuilder::from_installed_program(installed2)
            .autopush(true)
            .buffers(Buffers::OnlyRx)
            .clock_divisor_fixed_point(int, frac)
            .push_threshold(32)
            .in_pin_base(r.id().num)
            .in_shift_direction(ShiftDirection::Left)
            .build(smrx);
        // The GPIO pin needs to be configured as an output.
        //sm.set_pindirs([(led_pin_id, hal::pio::PinDir::Output)]);
        rx0.set_pindirs([(r.id().num, PinDir::Input)]);

        let installed3 = pio.install(&collision.program).unwrap();
        let (mut coll0, _, _) = PIOBuilder::from_installed_program(installed3)
            .clock_divisor_fixed_point(int, frac)
            .jmp_pin(c.id().num)
            .clock_divisor_fixed_point(int, frac)
            .build(smc);
        coll0.set_pindirs([(c.id().num, PinDir::Input)]);

        //let group = tx0.with(rx0).sync().start();

        let mut tx0 = tx0.start();
        let mut coll0 = coll0.start();
        let mut rx0 = rx0.start();

        Self {tx: tx, rx: rx, _tx_pin: TX::from(d),_tx_en_pin: TXEN::from(den), _rx_pin: RX::from(r), _coll_pin: C::from(c),txsm: tx0, rxsm: rx0, csm: coll0, pio: pio, /*masks: Mask::new()*/ }
    }

    pub fn write_read(&mut self, buffer:&[u32;8]) -> Result<i8,i8>{
        for i in buffer.iter() {
            self.tx.write(*i);
        }
        self.pio.force_irq(1); // start tx
                      //delay.delay_ms(1000); // wait for tx to complete
        while !self.rx.is_full() {
            let mut status = self.pio.get_irq_raw();
            if (status & 0x04) == 0x04
            //we got a collision
            {
                //info!("we got a collision");
                self.txsm.restart();
                self.txsm.clear_fifos();
                self.pio.clear_irq(0x04);
                self.csm.restart();
        
                return Err(-1);
                
            }
        }
        self.rxsm.restart();
        return Ok(0); 
    }

    //if there is a message read it out othewise return None
    pub fn read(&mut self, callback:bool, parse:bool )-> (Option<[u32; 8]>,Option<PacketParsed>){
        if self.rx.is_full() {
            // we have a recived message
            self.rxsm.restart();
            let mut buffer: [u32; 8] = [0; 8];
            for i in 0..8 {
                let temp = self.rx.read(); // read it out
                match temp{
                    Some(word)=>
                    {
                        buffer[i] = word;
                    }
                    None =>
                    {
                        return  (None,None);
                    }
                }
            }

            if callback == true{
                //let _ = self.masks.check(buffer);
            }
            let mut parsed = None;
            if parse == true{
                parsed = parse_rx(RawPacket{data : buffer});
            }

            return (Some(buffer),parsed);
        }
        return (None,None);

    }
    pub fn task(&mut self){}


    // /// add a new mask and return the mask index
    // pub fn add_mask(mut self,new_mask:[u32;8],callback: F) -> usize{
    //     return self.masks.add_mask(new_mask, callback);
        
    // }

    // /// remove a mask and its callback
    // pub fn remove_mask(mut self, index:usize)-> Result<usize,String>{    
    //     return self.masks.remove_mask(index);
    // }

    // /// keep the same 
    // pub fn update_callback(mut self, index:usize, callback: F)-> Result<usize,String>{
        
    //     return self.masks.update_callback(index, callback);
            
    // }
        
}




