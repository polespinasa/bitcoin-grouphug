//! Functions related to the transactions validation and manipulation.


use bdk::bitcoin::psbt::Input;
use bdk::bitcoin::{OutPoint, Witness};
use bdk::bitcoin::{Transaction, consensus::encode::deserialize};
use bdk::blockchain::{ElectrumBlockchain, GetTx};
use bdk::electrum_client::{Client, ElectrumApi};
use hex::decode as hex_decode;

pub fn get_previous_utxo_value(utxo: OutPoint) -> f32 {
    // Given an input from a certain transaction returns the value of the pointed UTXO.
    // If no UTXO is recieved back, the value returned is 0.

    println!("Connecting to the node");
    // Connect to Electrum node
    let client = Client::new("umbrel.local:50001").unwrap();
    let blockchain = ElectrumBlockchain::from(client);
    println!("Connected to the node");

    let tx_result = blockchain.get_tx(&utxo.txid);

    match tx_result {
        Ok(Some(tx)) => {
            return tx.output[utxo.vout as usize].value as f32;
        },
        Ok(None) => {
            println!("Previous transaction query returned NONE");
            return 0.0;
        }
        Err(_) => {
            println!("There is an error retrieving previous transaction");
            return 0.0;
        }

    }
}

pub fn previous_utxo_spent(tx: &Transaction) -> bool {
    // Validates that the utxo pointed to by the transaction input has not been spent.

    println!("Connecting to the node");
    // Connect to Electrum node
    let client = Client::new("umbrel.local:50001").unwrap();
    let blockchain = ElectrumBlockchain::from(client);
    println!("Connected to the node");


    let outpoint = tx.input[0].previous_output;
    let tx_result = blockchain.get_tx(&outpoint.txid);

    match tx_result {
        Ok(Some(tx)) => {
            // validate if the output has been spent
            //let spent = tx.output[outpoint.vout as usize].script_pubkey.is_provably_unspendable();
            println!("I'm here");
            let utxo_script_pubkey = &tx.output[outpoint.vout as usize].script_pubkey;
            let utxo_list = blockchain.script_list_unspent(&utxo_script_pubkey);
            println!("I'm also here");
            match utxo_list {
                Ok(returned_utxo_list) => {
                    if returned_utxo_list.len() > 0 {
                        println!("Transaction available");
                        return true;
                    }
                    else {
                        println!("Transaction already spent");
                        return false;
                    }
                },
                Err(_e) => {
                    println!("Error querying for the UTXO");
                    return false;
                }
            }
        },
        Ok(None) => {
            print!("Petition succeed but no tx was returned");
            return false;
        },
        Err(_e) => {
            println!("Could not retrieve previous transaction");
            return false;
        }
    }

}

pub fn get_num_inputs_and_outputs(tx: &Transaction) -> (usize, usize) {
    // Return the number of inputs and outputs from a given transaction in a tuple
    return (tx.input.len(), tx.output.len());
}

pub fn check_sighash_single_anyone_can_pay(tx: &Transaction) -> bool {
    // Ensure that the signature is using SIGHASH_SINGLE|ANYONECANPAY
    // The tx must have only one input and one output
    // Script must be simple P2WPKH (witness: <signature> <pubkey>)

    if tx.input[0].witness.len() != 2 {
        println!("Witness has more than two elements");
        return false;
    }

    let input_query = tx.input[0].witness.to_vec()[0].clone();

    match input_query.last() {
        Some(input) => {
            // 131 decimal representation of 0x83 designated to SIGHASH_SINGLE | ANYONECANPAY
            if *input != 131 as u8{
                println!("Sighash type not correct, must be SIGHASH_SINGLE | ANYONECANPAY");
                return false;
            }
            println!("Sighash correct {} is SIGHASH_SINGLE | ANYONECANPAY", input);
        },
        None => {
            println!("No witness");
            return false;
        }
    }

    return true;
}

pub fn validate_tx_query_one_to_one_single_anyone_can_pay(min_fee_rate: f32, tx_hex: &str ) -> bool {
    // Validate that a given transaction (in hex) is valid according to the rules.
    // Rules:
    //  - Should only be 1 input.
    //  - Should only be 1 output.
    //  - Tx fee should be bigger or equal than the min_fee_rate
    //  - The input cannot be spent before must be and UTXO.
    //  - Signature must be SIGHASH_SINGLE | ANYONECANPAY


    println!("Deselializing");
    let tx: Transaction = deserialize(&hex_decode(tx_hex).unwrap()).unwrap();


    // Only one input
    let num_inputs_and_outputs: (usize, usize) = get_num_inputs_and_outputs(&tx);
    if  num_inputs_and_outputs != (1,1) {
        println!("Number of inputs and outputs must be 1. Inputs = {} | Outputs = {}", num_inputs_and_outputs.0, num_inputs_and_outputs.1);
        return false;
    }
    

    let previous_utxo_value: f32 = get_previous_utxo_value(tx.input[0].previous_output);
    let real_fee_rate: f32 = (previous_utxo_value - tx.output[0].value as f32)/tx.vsize() as f32;
    if min_fee_rate > real_fee_rate {
        println!("Cheating dettected on the fee rate. Fee rate declarated {} - Fee rate found {}", min_fee_rate, real_fee_rate);
        return false;
    }
    
    // The signature type must be SIGHASH_SINGLE |ANYONECANPAY
    if !check_sighash_single_anyone_can_pay(&tx) {
        println!("Wrong sighash used");
        return false;
    }

    // Output not spent
    if !previous_utxo_spent(&tx) {
        println!("Double spending dettected");
        return false;
    }

    return true;

}




#[cfg(test)]

mod tests {

    use crate::utils::transactions;
    
    #[test]
    fn test_validate_tx_query_utxo_wrong_sighash() {
        let fee_rate: f32 = 10.0;
    
        //tx should be rejected because of wrong sighash type
        //tx id: d11251712c854dea5a05aed75c6d9d81aa3a51088d8031c5ecaa28afd2b277d5
        let tx_hex = "0100000000010109abff3c9bd88810da1dc5583e82834b612364c074799bfbbd1750bd29858888000000000001000000010f2700000000000022512039112e42819fe026c6c1406fa5c06646435ae2669bdc5b874234f72215c489fc03409ce5d98d03b32abc4af7dcc473811ff93e4f4e88e8ac0cf4b86831f491849f3db17fb916e732d9a84bc249a787d7a2c88dda6c0e5581faed9bd49b305a0ecb976d206af366cc2af6b6068e737543a26044363897cd492d58fc7055bdeb8eb494873dac00630461746f6d03646d743ea16461726773a46474696d651a65b9e44c656e6f6e6365190c9868626974776f726b636538383838386b6d696e745f7469636b657268696e66696e6974796821c16af366cc2af6b6068e737543a26044363897cd492d58fc7055bdeb8eb494873d00000000";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

    #[test]
    fn validate_tx_query_2_outputs() {
        let fee_rate: f32 = 10.0;

        //tx should be rejected because has 2 outputs
        //tx id: 3cbcf427d8f56dc4fe886b60324d4fa29387636705445dbaf6aa68629b0a46fe
        let tx_hex = "01000000000101f447a5f50615c2dccbe3d7cca44da2070a130af8a3a1d5cc305ce5548d108f7f0100000000ffffffff02972c0a00000000001976a914cf3f317c74a73afc94ac53bcebe21f42ff84731e88ac4443134b000000001600140d7b4a861b7158a10772bb04aeeb25ff9c8b393802483045022100ebe79839ebca5a467f02070c6873f89e903c308ca4b1c360f54b99a4bb4b7b5002201506e5b586dd2c2901a229301f256eaf765045c252a15d5cd3b2dd9b8b16c7fd0121024b0463b084e8db9b90d9af135ed6f7e6601caea1598a1271ee946454ac934c0c00000000";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex),false);
    }

    #[test]
    fn validate_tx_query_2_inputs() {
        let fee_rate: f32 = 10.0;

        //tx should be rejected because has 25 inputs
        //tx id: 674ababade440d16e474f3ee21485461bdcfbcc43b665544b9fa8ef54eb86f7e
        let tx_hex = "020000000001192fb35a1127aac1597315ba0fcaccfc0a6b736897c4a0a0a6ed2d673bdd38eb040000000000df0700003871ba3bdb970160026cd24ee8017b76ce5275c6df3924f8e71d60ed12c58f360000000000df07000019d1164c7617e77f2b2caf60c798121ceb6e07b23867872a0c9aab701b098b1d0000000000df07000043808f6d65ed46e8b587a3b8adc7b8a310a92823758e33817c466749fabb08080000000000df070000b1832c21e9137af01f988bd5ef0d9ccbb12515a80dcf7222cb44a4f723f31cec0000000000df070000009a1d80f90d96480956889a87629f6cdb2d4b2250e4a9b047585060c85af0be0000000000df070000434ebb6cceb60de03ec04a72c3b8bb877891596e1d9b423177b6ae2d2a7f1cad0000000000df070000d23d2a7980a4b3925a403871b553b8f2e88cf498f99cd4f5e3c98babadcbc71d0000000000df070000ad25e4df49d6581be5ae03a665001329d1ca20393c5a361ed321f0fd04784a3d0000000000df0700006563b0f523c552f1af77c1058982f1d8249b5a097b44462f6bedcee47f1b03740000000000df070000f78de8162134ae885732fc4b32d1b24d4d9d1acd53fc83e82bd4c8914cc42c3c0000000000df070000ff702fa2f5488f02443a57d09cf84be68796498871bf09d7470dc2dae264fb030000000000df070000c4b720025b3f36d9f9528f0c4457c13933fd4b6a4f5449b210326f9897294b560000000000df070000c037b5cd746fd54444b79518c3f5cb67a2fd46acafb69b18e398fc0fc6f6b9fa0000000000df07000021633f70559b3ddb4a59261664e23f7aeaccbc117a3d114a660006971b02e5a90000000000df07000068c7435cedf670c46da39cbc0fad0786d06749acfd3c6daddc09675f450e76fe0000000000df070000ebbe962525df30d905fd41f5480f1f3a2f1e650b68d3bc80703fe886e577d12b0000000000df070000799f260846fdb248fac190274181012fd4516288328fd1661e4bc00d698ec6300000000000df07000074fc091415a01207ab97ddeb91f764a91e61448ba251bbbac0e0612e9e286eba0000000000df070000741c98df591e528e77f63dfdee95a9af6b3f024f3d63b75cd51e88360fcc8bd70100000000df0700000018c213fd7b5c92e630528ba38b04c151774649a9dd7248161feb8856c9adf80000000000df0700007610047fd4446b6fbab5e3c0da6f18e362572077e5ef6dddcf2b89a02168d1560000000000df0700007d7d33b461eddfb3772c6c9cbd65a2216744cce3a9566bacfadb759f168ad12f0000000000df070000286e35bfc22fa88a4b77a3ca96de1a49de45fba86a8d2cd6acc4a2d65fd33a390000000000df0700007170394b29725b36f38309dd2c021c95a2373f1570a216a94f24f255857455070000000000df07000001e9bc930100000000160014a1c2c28987ccfacb1f12017bc491b88b5178bc910347304402200f1096787f653aea39ed8542332664d9db99730d05dd35a7fcac4e70658a1bb00220203642f78ddd21f5001ab13671f1c78e187741483292b7d20e37b5701cb3e43301004d63210245827098aaec677c99c2c505632d57930d138e28b98227a20c331dbaf08df4726702df07b275210268bd5a523bc6f2f3a741f03a13974a937db7060b5985723880572f2224071a9068ac03473044022019cbcaf55b15835941b383af066b480ee4eb686a6d061e265710f9668b8719af0220735bf4781354d0280b6593ed252c7952d2db777bc102436b0269cefcb2e495f901004d63210209edc0b1a15d130a4ed453308909a7673b20c978fc472868233a86244714f6a46702df07b2752102b30d3713f3c597a09ae233d09008b709c6c5c8076e426ee27db889d4b6575afc68ac03473044022065d2379ca31fee7b391a6cd57669c2d8bc9ae7c68c797f8dc96ea167ec0384ca02207bcce468d57423bd26fcafb2b152e7a3a3a4a2fc9fd76ccac51f688d1bdfafa601004d632103584a1fe6d921970c3bcef2954d8824aa87a003be6e6c54062c630532dc7ce0af6702df07b275210365819747aa0457f3251dce2f70cbf5cc5d5ec00c3e6ac0cbe6a3a71b86fdce6f68ac0347304402203d2d80ab8f00c9434fa3e1f79e17ea532713bc4472f2a0fbed6fa598a55d2a6202204031f1a6210a441650db7520a6ebebdc89be4df6d741bf0f8f34d619306f4c0201004d63210217c9b996fcd8efc8b360ba3e3aa8d4768f0085a1917a5b8dd18cf454e3c645706702df07b27521038b283305582cb2e7555c48afba5a5d64c139cae671f69cd10259059a2027c12568ac0347304402205e4243b266c45102a8f078fb7f310fde4e2139139f248b3c42c3365032cdad6602206005668b09495a9a5900d38407436de161f11a6c588568ab1c62641a3a0fe18c01004d63210264034a949b2aab9502394a6ae566c63d58b37dba53e8e2adc78d46caca07e5e06702df07b2752103d63f435a8055db20047d396656504c2c3e98cc2ebed59dcf3f9511f79fee432068ac0347304402207de4b187290cbd413a42f167186db645c1ae881bc72768ac4f2e01d7c9877d5102204363124187e2dd74ac93eced3b2fc8d7fcaa965fbc615468089533e62bbaa39a01004d632103242e51e2aa4f6cae00f1d819507fa7353d5b187095d2e2a1221bf9ed838d85e26702df07b2752102fea933a66ef1034865d3b1790b9111515482890da22d09a15b70d94a1193327468ac03473044022056208bcbfb45d5116f79c16ad2893720f65e0815b0b99179d376f06d5af2e29d022044cfcc0119e6c42fe42c26d3e4896eb7cecf6f205820601b83e20ff6a055a93101004d632102b930fac84933f2e54847e0cc56ebf392a4f9bff41edcb80b11f3ba3c28cb78db6702df07b27521025ef204f747f9e69ca742ac7f417668f3b1c8d59653a186de828e63d22fac3f0468ac0347304402200a2dea8f1010038bddf79e21fe467e743a7aed92e8c8427280a8c82209fabcff02201ef1cd2d03ac5ba3b380bbae52803a27ae35089194aeca5cf90af08ceb7cfddc01004d63210285d4ff30dcf42b50c8c107afb9b7793b148deef8395a030f6cc35c1b3cd325816702df07b27521025f202cfd0b46d5c2bb8f4fd061f7feb2d89df58034a89b8ee3175d2b0ae4798f68ac03473044022013379d83db24d27bdfb0da4303d0ba55393652bceeb44718ac67d9918a99d443022050917072fff42237e568c7ff94e92980da555aa0275a6d567b943d707fb9323b01004d632102e1e10398f41ccc96067fe6b770618f3ab49ca941c92fa8ed5403b39a2fc164e66702df07b2752102a963435b5e13ed01df6dd4a38af583404789f0ff723548e61efeffe2d8e73ead68ac0347304402203405574628d4a2f431a655af91824169668b8dee4c572e395399a7182b7cab8602206526954f79a282482b90f0237da228095acb3ef2b6033694959a7841bf5e62e001004d6321038652b07fa407a29d98cfa4958ca981b8e2ac6b7b229190e5157be67d559f9a016702df07b2752103bb56a77dcba5baa7eb33afe5d5f8233929275b2434c583952b13278a5461fdfe68ac03473044022032581b48eb0fa50628d8e1d662e61452ac008df836c14ca101d8cc12c44e048b022025fca64d817c1584c4ae67e68d58dcf57f90763e270efc51f05487a0e2e6fe9001004d632103364847d9d6ee5c02b1438a851779c5af826b4c9e896b11a57925db79934a55996702df07b275210264dd2878f48c8482b855d81e16a0566a5ab9d89c9011ae921cd79924992d028e68ac0347304402202955c60845c620b1c9a547e0380013b572323eb67895ecce1ae7dbe77f003ba502203baf5e53f89643dc1fb1d177d55f2c0d3115ed71bb0c46332a7ea48efe621b3c01004d6321028ee985e54a2d75c49a4aa8338ba4e8dd3e50b6bb26828eca26703f42ca55b92d6702df07b27521024a5b2b77bec1d91699459c44604911265d8bfdc6907f363afbb3ad1b2056dc1568ac03473044022057fdca8c0d938dd6ed34a0fd2885dfbfacf3652de8778b910a4747f0f90c898a02200dd39a9f679ab6162d00df9860c667a5189bef0ccf5bc4641d65cdf807c5dc1901004d632103d18c29361180e7497977ee722dae1f3e19077fd6d615c046c72fc1a2759c05556702df07b2752102c58f565c0a2932c4243727c8bfc0021e21d2e4de34e1f96d3d2ee2694550696668ac0347304402203871497f9fb4ba247360cde53b4269fac4e4875a861f97180405b10e2828399502202c4d8d39c9a839fd3a1600fb8af702da7d9a2a568cd356ed7983c8dccc3d51b101004d6321038b02c41d2b4db54af112954b7c40b4d086831856c7c5bc38d34f619002c2e1876702df07b2752102804aaa4592299b1e98aee463e7eadf04cd7f788c8b546ec82fce1c331188c20c68ac0347304402202cf707a9da568dd17fbae968a8b96d5df07c3c75a996287eb38f50047c0211430220332de55c30b0ad8633048d8d58d4ed64ae7ad60ba5b9ead487a16f7a812d31bf01004d632102371238e7d0802227ed90558a1c5615575bf96241c2c56e6030be4d444255f45c6702df07b2752103b9f11e3ffa7de8c7c2bd2fe51023385b82d8ea66605285984938a3f6d665fbe568ac0347304402206ed592296fab622fb9352cabd4230a9fe2efd6e29445bb012868fd0a5a77bd020220500f4798ff5819b902106b35cf74044e41692771d98b94169f2b0ded0e1aa4c701004d6321031cdb7f68be6bb34d9481f9b6539eb19564a4180af032c3b5a037b075505938f16702df07b2752103ad4c5462c303ab3e3e8d81de2bdaf32e5f688cc9db976014fae065115d24396c68ac0347304402205673806a4c3dba6529ef8af339b43364f26355e2b7b4ac41aa5841f8349dafea02203e21dbaf9ba8d6c3edb0ed575997408bd9b5a6e831a04baa557b1456da9bcee901004d63210229d7bc33a0b168291b4244a539b34a674ca908e522ef75d812927f1539c80b796702df07b27521025d763d65d195bb00604b3099da287ea1b7a5711f5445636f5abb308de9d08db468ac0347304402207bfd64d9290a56bfd88f5375f7f9db9a15abd6ca9f91696937cd62bd336396450220233ae649844d179b56988b1d204b61f536b2d41798fc3cd444429999f159189a01004d632103e5cfb587fa693edb1922895c9598f54b5a04a385017b470c681a49359e709c0c6702df07b27521020de4a28af58bcfdff1164b9b05555e50cf47a205110fd2a3de70912310a7e58468ac0347304402207cbd53df448503f0e74701a0734bb2bf4488c82231db3fc3c7fb73a6524ab7180220478b1eb077d5442ef1d956a2fe6d9d33648038d805265f025f0a88972f3e04e801004d632102af6a0dd96954f02157daeb7c306a976049c34a33f8b7e74d61616aaffd3075246702df07b27521030c06338239e04f19f4abff41557a4d658533e1bc566aa5c55feb6638a1caf5d368ac03473044022071ec716398c0cf146c5c42bd63bd6664a6811530f9a817e76f56f4b98b40e245022054eb012c7c599e1d9f3f1b3566d4c3f5bea897b7c881969528c8548542abfccb01004d632102175a0fbf6c8d68891b14e5ba7ae2893a70a7e55d59063f422a619323e2c297dc6702df07b27521020df0cf69051c9a4d4f88b0ec2bcbae5c7a2c163295e3bac6fe34afd66e9e529268ac0347304402204ad06772296c5260a6dbc216d8098b2657e7c7a20b47ca7b502464214abe51b602207345eb2856dca68424d8eb8c50360f7c0e1435861d6e09689fbde6ffa68a728d01004d63210246940e767994bcd5253688b39fb4c7c80c7e2feb130cf7c1c91bf348ffefd0be6702df07b275210272fd7e9bace91c1f6c15bbbfdb5e5d6033d30309cc112b08b61b15e8811fe84068ac034730440220738c995f0bfaf9f85b219be2b81d3cadb39948516aebff0c4641d068408b99dd02200cd2f43b45d44f2f0ef3963251450c74369526424520412f2efd3a0df11bf48b01004d632102ea2d33079cc543d0403eab89c9f2acfb3230c52a325d741ecd5fcbbea05af4d96702df07b275210305e8868eaa326708fa92a34f51e9f0fbe6318888b1a7da250188d071addb1e5268ac03473044022012d19437c3d4bdc9cf15c4c7d40f3301d2539dc9bd2c64b3d3d34c3256e5e02d02207c6bcbdb4b43c47cec9fac268a12d8bf33a917ca0425cf656f197324e846b21601004d63210212d417c4a573397bc32d90a4f082b9c6a32865ba84cf95fa55771644634cd2646702df07b27521037719811f4b5fa85aaabe0dea307253d958c16126c15a6a9979c97e9a7b2b6f0068ac034730440220153b6ba96272e2a3cd900e4cb4fbfa820016f9eff508834231c9d3780abc1a9702204b991271b4d4a75c22e369c1e280d588339e9a131c934b16100ef4a4f560a81101004d63210267bb5279ed0fbd1b89940de409c15814ffe3290528aa0f1c53ef6414cdaab37b6702df07b2752102be80756e49599fd8a92f6dc1f8aa7564e29d8ffb3cb6784da8b941439b2ea57d68ac0347304402201f08702170ebe5a74952d037133fcff8e62b20a45b2f519d932152e34a8560a7022062cdd45b0e03c0aa053a6dafdb8dd363677bf1656ed8e8e8b596933de58445c601004d632102c28bedd521765b4f89de2c95b0ffaeffbc8eb3a40ab96dcb09d3f89359e4e89c6702df07b2752102f33fbeff7de4f65609e7873247cbd74795a2c0dc6e3ec840513ac7ca9cd7f23768ac00000000";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

    /* #[test]
    fn validate_tx_query_valid_tx() {
        let fee_rate: f32 = 10.0;

        //tx for this tust must satisfy all requirements
        let tx_hex = "";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), true);
    }
    */

    #[test]
    fn test_validate_tx_query_fee_to_low() {
        let fee_rate: f32 = 70.0;
    
        //tx should be rejected as real fee is below the declarated one.
        //tx id: d11251712c854dea5a05aed75c6d9d81aa3a51088d8031c5ecaa28afd2b277d5
        let tx_hex = "0100000000010109abff3c9bd88810da1dc5583e82834b612364c074799bfbbd1750bd29858888000000000001000000010f2700000000000022512039112e42819fe026c6c1406fa5c06646435ae2669bdc5b874234f72215c489fc03409ce5d98d03b32abc4af7dcc473811ff93e4f4e88e8ac0cf4b86831f491849f3db17fb916e732d9a84bc249a787d7a2c88dda6c0e5581faed9bd49b305a0ecb976d206af366cc2af6b6068e737543a26044363897cd492d58fc7055bdeb8eb494873dac00630461746f6d03646d743ea16461726773a46474696d651a65b9e44c656e6f6e6365190c9868626974776f726b636538383838386b6d696e745f7469636b657268696e66696e6974796821c16af366cc2af6b6068e737543a26044363897cd492d58fc7055bdeb8eb494873d00000000";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

    #[test]
    fn test_validate_tx_query_double_spending() {
        let fee_rate: f32 = 70.0;
    
        //tx should be rejected as real fee is below the declarated one.
        //tx id: d11251712c854dea5a05aed75c6d9d81aa3a51088d8031c5ecaa28afd2b277d5
        let tx_hex = "0100000000010109abff3c9bd88810da1dc5583e82834b612364c074799bfbbd1750bd29858888000000000001000000010f2700000000000022512039112e42819fe026c6c1406fa5c06646435ae2669bdc5b874234f72215c489fc03409ce5d98d03b32abc4af7dcc473811ff93e4f4e88e8ac0cf4b86831f491849f3db17fb916e732d9a84bc249a787d7a2c88dda6c0e5581faed9bd49b305a0ecb976d206af366cc2af6b6068e737543a26044363897cd492d58fc7055bdeb8eb494873dac00630461746f6d03646d743ea16461726773a46474696d651a65b9e44c656e6f6e6365190c9868626974776f726b636538383838386b6d696e745f7469636b657268696e66696e6974796821c16af366cc2af6b6068e737543a26044363897cd492d58fc7055bdeb8eb494873d00000000";
        assert_eq!(transactions::validate_tx_query_one_to_one_single_anyone_can_pay(fee_rate, tx_hex), false);
    }

}