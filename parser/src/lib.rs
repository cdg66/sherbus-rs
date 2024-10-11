#![no_std]



pub mod parser
{

    pub const PACKET_SIZE: usize = 8;
    pub const MAX_PAYLOAD_SIZE: usize = 32;

    // raw data from the PHY.
    pub struct RawPacket {
        pub data: [u32; PACKET_SIZE],
    }
    //type of data payload
    pub enum PacketType {
        NoType,
        Control,
        Interupt,
        EchoRequest,
        EchoResponse,
        Stream,
    } 

    // separated feild of a packet
    pub struct PacketParsed {
        pub protocol:bool, 
        pub packet_type:PacketType,
        pub from:Option<(u8,u8)>,// leave none and the feild will not be added to the packet
        pub to:Option<(u8,u8)>,// leave none and the feild will not be added to the packet
        pub appcode:u16,// 
        pub payload:  [u8;MAX_PAYLOAD_SIZE], // in u8 cause payload may not 32 bit aligned
        pub payload_len: usize,
        //payload[0] is MSB and the code fill it as much as it can and discrard the rest
        
    }
    //# hey
    pub fn parse_rx(packet: RawPacket) -> Option<PacketParsed> {
        let mut packet_parsed:PacketParsed = PacketParsed{
            protocol:false,
            packet_type:PacketType::NoType,
            from:None,
            to:None,
            appcode:0,
            payload:[0;MAX_PAYLOAD_SIZE],
            payload_len:0,
        };
        let packet = convert_u32_to_u8(packet.data);
        if packet[0] & 0x80 == 0x80 // are we in the spec
        {

            //Set the Recived message Type
            match  (packet[0] & 0x7C) >> 2{
                0x10 =>  {
                    packet_parsed.packet_type = PacketType::Control;
                },
                0x08 =>  {
                    packet_parsed.packet_type = PacketType::Interupt;
                },
                0x04 =>  {
                    packet_parsed.packet_type = PacketType::EchoResponse;
                },
                0x02 =>  {
                    packet_parsed.packet_type = PacketType::EchoRequest;
                },
                0x01 =>  {
                    packet_parsed.packet_type = PacketType::Stream;
                } 
                _ => {return None;}
            }
            if packet[0] & 0x01 == 0x01{// do we have BPI
                packet_parsed.from = Some(((packet[1]&0xF8)>>3,(packet[1]&0x07)));
                packet_parsed.to = Some(((packet[2]&0xF8)>>3,packet[2]&0x07));
                packet_parsed.appcode = ((packet[3] as u16) << 8) | (packet[4] as u16);
                //#TODO copy payload
                
            }
            else { 
                packet_parsed.from =  None;
                packet_parsed.to =  None;
                packet_parsed.appcode = (packet[1] as u16) << 8 + packet[2] as u16;
                //#TODO copy payload
            }
            return Some(packet_parsed);
        }
        // we are no in the protocol so we just copy the payload without processing
        packet_parsed.payload = packet;
        packet_parsed.payload_len = MAX_PAYLOAD_SIZE;
        return Some(packet_parsed);
    } 

    // utility function to convert a u32 array to a u8 usefull for rx
    fn convert_u32_to_u8(arr: [u32; PACKET_SIZE]) -> [u8; MAX_PAYLOAD_SIZE] {
        let mut result = [0u8; MAX_PAYLOAD_SIZE];

        for (i, &value) in arr.iter().enumerate() {
            let base_index = i * 4; // Each u32 value corresponds to 4 u8 values

            result[base_index]     = (value >> 24) as u8; // Most significant byte
            result[base_index + 1] = (value >> 16) as u8; // Second byte
            result[base_index + 2] = (value >> 8) as u8;  // Third byte
            result[base_index + 3] = value as u8;         // Least significant byte
        }

        result
    }


    pub fn parse_tx(packet: PacketParsed) -> Option<RawPacket> {

        let mut packet_to_send = Some(RawPacket{data: [0;PACKET_SIZE]});
        if packet.protocol == false { //if not a specification packet send the whole payload without further treatement.
            
            match packet_to_send {
                Some(ref mut packet_to_send) => {
                    // Fill the array with specific value
                    packet_to_send.data[0] = (((packet.payload[0] & 0x7F /*force P bit to 0 */)as u32) << 24) |((packet.payload[1] as u32) << 24) |((packet.payload[2] as u32) << 24) | (packet.payload[3] as u32);
                    packet_to_send.data[1] = ((packet.payload[4] as u32) << 24) |((packet.payload[5] as u32) << 24) |((packet.payload[6] as u32) << 24) | (packet.payload[7] as u32);
                    packet_to_send.data[2] = ((packet.payload[8] as u32) << 24) |((packet.payload[9] as u32) << 24) |((packet.payload[10] as u32) << 24) | (packet.payload[11] as u32);
                    packet_to_send.data[3] = ((packet.payload[12] as u32) << 24) |((packet.payload[13] as u32) << 24) |((packet.payload[14] as u32) << 24) | (packet.payload[15] as u32);
                    packet_to_send.data[4] = ((packet.payload[16] as u32) << 24) |((packet.payload[17] as u32) << 24) |((packet.payload[18] as u32) << 24) | (packet.payload[19] as u32);
                    packet_to_send.data[5] = ((packet.payload[20] as u32) << 24) |((packet.payload[21] as u32) << 24) |((packet.payload[22] as u32) << 24) | (packet.payload[23] as u32);
                    packet_to_send.data[6] = ((packet.payload[24] as u32) << 24) |((packet.payload[25] as u32) << 24) |((packet.payload[26] as u32) << 24) | (packet.payload[27] as u32);
                    packet_to_send.data[7] = ((packet.payload[28] as u32) << 24) |((packet.payload[29] as u32) << 24) |((packet.payload[30] as u32) << 24) | (packet.payload[31] as u32);
                }
                None => {
                    // impossible
                    return None;
                    //panic!("packet_to_send should be Some but found None instead ");
                }
            }
            return packet_to_send;
        }
        let mut control_byte:u8 = 0;
        let mut bpi:u16 = 0;
        match packet.packet_type{ // set type 
            PacketType::NoType=> control_byte  = 0b10000000,
            PacketType::Control=>control_byte = 0b11000000,
            PacketType::Interupt=>control_byte = 0b10100000,
            PacketType::EchoRequest=>control_byte =  0b10001000,
            PacketType::EchoResponse=>control_byte = 0b10010000,
            PacketType::Stream=>control_byte =       0b10000100,
        }
        

        if let Some((position_from,subdevice_from)) = packet.from { // unpack the the from tuple to see if we need to add bpi
            if let Some((position_to,subdevice_to)) = packet.to { // unpack the the from tuple to see if we need to add bpi
                control_byte = control_byte + 1; // set the last bit indicating a BPI
                

                //set the bpi 16 bit feild
                //let test = (position_from as u16& 0x1F)<<12 ;
                bpi = ((position_from as u16 & 0x1F)<<11) | ((subdevice_from as u16 & 0x07)<<8) | ((position_to as u16 & 0x1F)<<3) | (subdevice_to as u16 & 0x07);
            } else { // we got a from but not a to so we return none

                return None;
            }
        } else { // we dont get a from so its an 8 bit arbitration plus an appcode
            match packet_to_send {
                Some(ref mut packet_to_send) => {
                    // Fill the array with specific values
                    packet_to_send.data[0] = ((control_byte as u32) << 24) | ((packet.appcode as u32) << 8) | (packet.payload[0] as u32);
                    packet_to_send.data[1] = ((packet.payload[1] as u32) << 24) |((packet.payload[2] as u32) << 24) |((packet.payload[3] as u32) << 24) | (packet.payload[4] as u32);
                    packet_to_send.data[2] = ((packet.payload[5] as u32) << 24) |((packet.payload[6] as u32) << 24) |((packet.payload[7] as u32) << 24) | (packet.payload[8] as u32);
                    packet_to_send.data[3] = ((packet.payload[9] as u32) << 24) |((packet.payload[10] as u32) << 24) |((packet.payload[11] as u32) << 24) | (packet.payload[12] as u32);
                    packet_to_send.data[4] = ((packet.payload[13] as u32) << 24) |((packet.payload[14] as u32) << 24) |((packet.payload[15] as u32) << 24) | (packet.payload[16] as u32);
                    packet_to_send.data[5] = ((packet.payload[17] as u32) << 24) |((packet.payload[18] as u32) << 24) |((packet.payload[19] as u32) << 24) | (packet.payload[20] as u32);
                    packet_to_send.data[6] = ((packet.payload[21] as u32) << 24) |((packet.payload[22] as u32) << 24) |((packet.payload[23] as u32) << 24) | (packet.payload[24] as u32);
                    packet_to_send.data[7] = ((packet.payload[25] as u32) << 24) |((packet.payload[26] as u32) << 24) |((packet.payload[27] as u32) << 24) | (packet.payload[28] as u32);
                }
                None => {
                    // impossible
                    return None;
                    //panic!("packet_to_send should be Some but found None instead ");
                }
            }
            return packet_to_send;
        }
        // we got a 8 bit arbitration + bpi + opcode
        match packet_to_send {
            Some(ref mut packet_to_send) => {
                // Fill the array with specific values
                packet_to_send.data[0] = ((control_byte as u32) << 24)| ((bpi as u32)<<8) | (((packet.appcode & 0xFF00) as u32)>>8);
                packet_to_send.data[1] = (((packet.appcode & 0x00FF) as u32)<< 24) | ((packet.payload[0] as u32) << 24) | ((packet.payload[1] as u32) << 24) | (packet.payload[2] as u32);
                packet_to_send.data[2] = ((packet.payload[3] as u32) << 24) | ((packet.payload[4] as u32) << 24) | ((packet.payload[5] as u32) << 24) | (packet.payload[6] as u32);
                packet_to_send.data[3] = ((packet.payload[7] as u32) << 24) | ((packet.payload[8] as u32) << 24) | ((packet.payload[9] as u32) << 24) | (packet.payload[10] as u32);
                packet_to_send.data[4] = ((packet.payload[11] as u32) << 24) | ((packet.payload[12] as u32) << 24) | ((packet.payload[13] as u32) << 24) | (packet.payload[14] as u32);
                packet_to_send.data[5] = ((packet.payload[15] as u32) << 24) |((packet.payload[16] as u32) << 24) |((packet.payload[17] as u32) << 24) | (packet.payload[18] as u32);
                packet_to_send.data[6] = ((packet.payload[19] as u32) << 24) |((packet.payload[20] as u32) << 24) |((packet.payload[21] as u32) << 24) | (packet.payload[22] as u32);
                packet_to_send.data[7] = ((packet.payload[23] as u32) << 24) |((packet.payload[24] as u32) << 24) |((packet.payload[25] as u32) << 24) | (packet.payload[26] as u32);
            }
            None => {
                // impossible
                return None;
                //panic!("packet_to_send should be Some but found None instead ");
            }
        }
        return packet_to_send;
    } 

}


#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
}
