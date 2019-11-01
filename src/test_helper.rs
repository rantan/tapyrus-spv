// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::store::OnMemoryChainStore;
use crate::chain::{BlockIndex, Chain, ChainStore};
use crate::network::Error;
use tapyrus::blockdata::constants::genesis_block;
use tapyrus::consensus::deserialize;
use tapyrus::{BitcoinHash, BlockHeader, Network};
use bitcoin_hashes::sha256d;
use hex::decode as hex_decode;
use tokio::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// A hundred block headers hex string.
/// Network is regtest and start with genesis block.
/// These headers has transaction count(it's always 00) at end for sending in headers message.
pub static HEADER_STRINGS: [&str; 100] = [
    "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4adae5494dffff7f200200000000",
    "0000002006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f9ac8c52a76cecbb7e642bf3456088cf92fda32cd746364180a058640c23b6e5ae4db3f5dffff7f200100000000",
    "00000020a59485ce0c710919c8b08b6c821c3ee9c8ac9f9f4e669865b2545aec7d03357631db0c7f7d68c7c479f68a015d0817b6530e451656b524c9e9e92fad8c0fb18711dc3f5dffff7f200100000000",
    "00000020d5e3096fd8b3e62cda15efab068a03d5638dea830125401b22a449fee5642f10d5638982537da6ed5fbbc305d6fbe9939bdaadacc1e5050b4253afc5349fd94911dc3f5dffff7f200200000000",
    "0000002078e0dd192118b729ef3fcae8c652a08411149cb041c341b71b804d943e7eed7355c080d5afb9947a0813fcf5d094a953227a5cf14706ca1eaf0dad1a1df51d3412dc3f5dffff7f200100000000",
    "0000002026fa5a5d583aeee50166aee3fd23b1be12d70109b9ca6894024a312f83f805625b2183b24650430f488dc934a7a420de0001024000625c1051d9c2bd702cfc8c12dc3f5dffff7f200100000000",
    "0000002072aa3f1d9a93fb465bee015fbe13f47530e55698b3cf86b95535d263f1a2e90cfb09a081a82f8dc8c7d618f6a3c2b6dabb16e3f8b7fcbf3feab0ef5a2c0bc5f012dc3f5dffff7f200100000000",
    "000000204b68e7945e82d78a7997256c3220811de64efbc66bffca0ac39ae5ab3ace627a5ef57b2bc5a5ba1c22fb9290284440813beef3ebf24bb6d9655a36ceadee450612dc3f5dffff7f200100000000",
    "000000208cbf05f78a146c5fbfa0028b8590c3452e580ba1db76a1967a645527237d2a09ed4e2e35533fd1cfe4e78cd7eb712aeb303adacc43d15cd577e7fff025fe410d13dc3f5dffff7f200300000000",
    "00000020b10895e1ab8a61f8b6fce5e5fa75d8e55463b1da94d561c04a9acc98c2395605275e585b973409554e40b925edf46659f71acac0688e7799e0b60b4de578015113dc3f5dffff7f200100000000",
    "000000209e7b1087ff87e006a1ab109c61d8f8157867cda933b9d797befd968872e033687f2ddca40f598c8e98b79eea6f757bb8bbd55ccdba3338199a7745d2ecfd7e5013dc3f5dffff7f200000000000",
    "0000002084b25d78733ab3c60f51964cfbb782d79dfe98bdcc8224b205c6e7c1a544e82ff285f6eec2249e58a26f0a99d73db9fa0ea09ff617df7629200936564170e7ae13dc3f5dffff7f200000000000",
    "00000020bbccaa60175228cfe8243304439c7cab303034d8e70d4bd1442edf667677ce68ee7714b2eef52375473f50a492fdd3794ba42ff11ac30055d9c11fb32f85686913dc3f5dffff7f200000000000",
    "00000020e6c1f21841685054bebe8c0a24237fe7dfb26faca259531435c608af80219701e9faf83ebabe8a48dd2e80895633e0b0826de8fe41c79bc3f78a99b4bb645acf13dc3f5dffff7f200200000000",
    "0000002063ef903231cd2e1e6f7487ee7ecaecaf3aa83f1bb539ce5bffd05ed275410e6db9acd16169cb3a356b04a14311bd4d385ef16802b34648da5cef91a0d572b1f914dc3f5dffff7f200200000000",
    "00000020783e043f3aea75a6c0b20dc611c6f546a9aea76f63660716a05e2740502bf50d3acd80a6d1e5105ca88b302219e5ddd670e406b192d7ee5fb838aef513f0bfe414dc3f5dffff7f200100000000",
    "000000200b6bf18bf4cc30db1ca983e8bb72074bba6b2ed6626e0a6add5c781bd763031e7a8bf90438161abe772d0f24f9a7f6dfd6bd923602b24e13334b1ba44b41dcd514dc3f5dffff7f200000000000",
    "000000205ed34f17407d5e225a89b5d5b4dab943bccca31e0c944d8927cd6b01fbf04f3ab55e6c7568dde00cbbaa885e496aa6aeafe78c09441bc8010186a68a7ddf0dbe14dc3f5dffff7f200500000000",
    "0000002008536a8608b4b25fc356d75031a17dcb7a0d9f1e4bfa96b0b37b41d0d38a4344f2f1c0cd541eb0c8b9f5151a3007da0ec2f231b6185faf0492d75f18efc2cdcb14dc3f5dffff7f200000000000",
    "000000206eb61e90a9e62855a4bf2a5f003203565ff7c25bd007fc4653a6edb897f3451acda8e18913f17995cc152d59fcef3ac18b02ae822067ef7a490cc833000bce0e14dc3f5dffff7f200300000000",
    "00000020b1c009f3edcc5159b6f6b632b433e773a888e695159151522530742afbbef329a18922d2926c764abe774b7cb507f17c32739331471082c8b264ba76f63140d115dc3f5dffff7f200000000000",
    "000000203cea71bc68a662f0e0c8e6c363a85b1c6707e8716eed54efefa47785e6621823ed79df9de6c7607c895cf04ccd014321362ee816d0505b0cdc6952c46b8d04fa15dc3f5dffff7f200300000000",
    "000000204d17823d8341381a52f3bd70db90fccf8dd85230ffb19ffb419920f19ae1fd6a962b3178f66ab073457a71da5e073a7d5818bc25f74258d4fe13c25a213519df15dc3f5dffff7f200000000000",
    "00000020a44fe78d0e055de067423e6a9bf44ad52a60405979588afdefd222a846c7ef45ae8586935f28d502978e146236cec0480a81bd575a3ff46dc23e9e0efe532b0915dc3f5dffff7f200700000000",
    "00000020a3aea44648bb23bbd124a66fb031f14a7118d041d18e2b8c3d3ad844ecffe17fa3396c08c9ba73bf3818ae74689470fd273b27a6618d9424cd07de030c49d05815dc3f5dffff7f200000000000",
    "000000201e0840750a0230e99c980ed891ea2b822ee79cdd6631b074ec1f26349f495357617cbecc24429f3e5e01456ee4d7bd8058b2c90023e789ffe9c96afb0310752115dc3f5dffff7f200000000000",
    "00000020b814bc4964d19e415dab6c5678f846bff979e16d83f47973c157843cf5f2f862f50454fb9c8e4fe31c2c8beac27a31879def6115c4c70bf7493323d5ee975cb516dc3f5dffff7f200100000000",
    "00000020ad253e6dbbdbb439802bdc7a46554efeb6a83ea3c8ade1611dfb2e720d3273545e55e72d1b1407f04825c3525cb11f05a0d7363899b87a702eaf63261162ae7216dc3f5dffff7f200000000000",
    "00000020692d91531cfb8549dc359923ab725f8c4f495bc8cbfd0355f9e0a588416d3811b485822a9a5fdf4613f7d03f98cd6d08a43ca3196324e219c9b27a3707fdf66f16dc3f5dffff7f200000000000",
    "00000020f783e2d0cbbec7f75515a694d54b809a8261ec0bc265a37cf00025954d47b469551cf4fef7df7ed1b5f304c9f5e3be89e1af56ee7f0290d07ac8b700020b76eb16dc3f5dffff7f200100000000",
    "00000020b51a710c49a420baa86c95289c872adb38b87ebff70deeb952ca1fd775736a7e7c200fc87895315c81e93c6234917fb199d696a2781a8ce2d6fecb6a89f9847e16dc3f5dffff7f200000000000",
    "00000020875cd2220c3b1735b5432dd73f41c3368a6854ac931e3cf88c0043749dad3b625e411ca081acf2d6c1c9cd4f07177cdde743980bc15bac52710fb3df9d2dbb6016dc3f5dffff7f200100000000",
    "00000020a3eaea6a310bdb6aaf3c370b693007705904981c6aa497ff00ac18d9bb2d600edfa0ce29fdfe6916b8f8194beafd9b01c0b688a66b4ce38e35de1073be8b5d0f37dc3f5dffff7f200000000000",
    "00000020e8fda7125438e3110878d74f7eef1aa8564d36df924a3c8a7ee722598b195f77e5ccea33a1c05a0bb560ff1c6d230f540a440bd4bc04c494b0fcac0e41178a7337dc3f5dffff7f200200000000",
    "000000200e19d4560264d15094338396f383b95a2837a793fab3bbc1b1ff579eca6db22218d837f34ab7712c83577856a4ad3d2cdc635ca5ba123f3bb95a829650f7f8ee37dc3f5dffff7f200000000000",
    "00000020a2cd2db2bb18a60b65dc23419b86c548eec7eabbda71a147d41a400f9b39f6728034f62c9c7e50ee19d7bf91b53ca3fd00c6c70ba920f842947ca09030ee713f37dc3f5dffff7f200000000000",
    "00000020dedef966899a676b7264892cffcae3c87f72321816fc28020b8102f8e08c3846c2749a4bba166badacc013362afd3ee31b4a56d81328529f3cb2ea248037ab6c37dc3f5dffff7f200100000000",
    "00000020344de86e5a71fbea7f6f13a2c4e7c9eb601867559b1e33c174e235b17b6d5f658f63a7748d1020dbbca349ebad18e9a49c860643ad2a4c9e1d0d93ef0216135237dc3f5dffff7f200000000000",
    "00000020eb38bdf3eac54d803839675bfc6894ec49a8cd5fdd48455d3ebb4c602cb6da2a49e4575f187bb5aedcee7f908b67aeea3f1ce25a1d6bc1ddb5064d2af6e43a3d38dc3f5dffff7f200500000000",
    "000000203611ad13ea2ca5be24706bad0e0bc3005157f83bc7d4c8d80e237c7490b5461db064ae05ab95aa22240568698c30c191e25c6e1a17e2c5951a1499e5dc260ad038dc3f5dffff7f200200000000",
    "00000020f8c725c5b4b175550e972dd76371eb65c6338e8b4ecbc5e670bd7b2e8ad75a0474ad1a72f5331bccb5bc63a0478f4c18fe674675e277bab0d85c49a90f26c4b338dc3f5dffff7f200000000000",
    "00000020f36633715cfec79a39fc46ad28fedfb593508566ca1bc810454c7a3c57bfff0af5809673193122df96b54c6ad21ff8747fc1bc0dcce3317a45cc0b7d4a95f20438dc3f5dffff7f200200000000",
    "000000200bc2722b9318ebca9a620745ecc67324991a98325cd19c08e83367827e981519d6ba7fb047dcdb83f5f73a0eb173a654348013de46d83775aa6c4adebb8cfe8938dc3f5dffff7f200100000000",
    "00000020c34572674211561670b3498d44655f4fd9da6e72ce45bb6142e853a94a00b957a3a0b079f18ee0aa6c7ae065c535c45330d6c333be5d5c3b032eda385782b25938dc3f5dffff7f200300000000",
    "000000200214d41948d1f662f711b5796e489b0ac0eb4b59dbbd58e8466e1738f8b99c27338cce8fcd511ecfdb6aa53becc7c41355a761e2110a94a95482a27cf5103bd739dc3f5dffff7f200000000000",
    "0000002096098d3c203b99663be0096bf2da44b67e598c30f0bd5b79c0b67cad57233010654e65e47424cb3565935b91712f6979d3467923f0faa5e2a00a0af5ac93330239dc3f5dffff7f200400000000",
    "00000020c99dfb2a230a5d22f21b6f98ed5ad74299e15e0ba4e5b75278d6fe6c6fca4e395755a6f7e6e601a97f924aeddf93aac694e58e0a581439c877a94517b354ab9a39dc3f5dffff7f200100000000",
    "00000020c110be6c24bdfdfc3e50cbf1e1b5f857bbeeb7394973e9ca6725486547acf12e5b09453408a5782a5c21891dbd344c6aede5210fa1a029a76f2870589a1a6b4439dc3f5dffff7f200300000000",
    "000000207b31d2e181894c6955b7aa924caaf59d521cecbbc0b22621e29059163772dd683a80b6c80354d2a1c6b513f12bec3c8f35477e770886c6454c2384d24505dfb639dc3f5dffff7f200000000000",
    "00000020a8ccaf8a3c7a468f223eb50c0435665b95c619cb8eef168eacaf3a5623d09d11e059eb69367aa2e656ae5e504c8b3b1c5fa7bd9c55f7ebfd7ba54abba49fba4939dc3f5dffff7f200e00000000",
    "0000002039da28572267fae01abfba7c64224840ae1883dbdb1c0d1664d07ede676b17012bd6eac0a2ca8ac1d2576c61ec1d70c8d4bebaaf7033339648cfe706ab9ab56e3adc3f5dffff7f200100000000",
    "000000206f6a6009fc3640eb67146f8b58981584f50054258bda8759f7469035eac1287b2b03f2afc0ab6ef1134f142734fb506b8f425ec29c817520a2d9b209c913812c3adc3f5dffff7f200100000000",
    "00000020b9f2d5e38482ca7a40e86e16ed819a62a303d188998c228ae397c1f27076ec74f674ba291b194de0f69ec63b3c8d1f5b38b5fe866ad4b5494d04b61f0122ff923adc3f5dffff7f200100000000",
    "00000020a69b7115c393819af658d05456ff12c50fd7b2a4535f92114121ae0dd0cc2f1040b0e8a2295626c3eff5dfd6eb6e773f42a13b2ac85da68f30264cf11335a4323adc3f5dffff7f200200000000",
    "00000020f7f8790f731ed4da55e9f74356a0842cc8f2d9909f872ba85554ed151c2ef64ccc140fc95ea13f1f31d61869c9888ccd7b2822fc5f0ee0c69e5a9a8420ecde8a3adc3f5dffff7f200000000000",
    "00000020fc974724c74650bc6570502ba8af3fdf39599a1045feddaf45d7e17d7b2b645ec6e0d65fe33c878ea01fb10bf1f7c2d1eb717ef0a5334d64d304a1a82dc8383a3adc3f5dffff7f200000000000",
    "000000203cad9ec12d4bda562daabdc2bd71af683dbc498413f2bba6c21554b04ace4c69d078e951f17701238a4079eb91b6e5efbdd52cebf049b5ed8328dd2fa22d9ce43bdc3f5dffff7f200100000000",
    "00000020fd107d7b8453846aab15c6d10cd93bf93587e5d588239c1b0a7b782a615aa465af727cf07a3573edb38e59ea5b075342f6bf969f8d39f3a4b8de4f266cb49b5d3bdc3f5dffff7f200000000000",
    "000000206389cb354fbd06a01fa81b592d4e289da0f1eb022a2ae533eb0e21b890238b6617c6e217420e1ede166893eb9cfddd26c553621ce1e77bb7dfee6c9fdf56978f3bdc3f5dffff7f200400000000",
    "00000020c699e45ea0df8d5359500c19414e3f9296c60e06effaf126879fe94a98b12559effc4d59e2cf7ae997a967d39e39869549848f86fcf5d197ded28302e8e39d853bdc3f5dffff7f200100000000",
    "000000209be5ae8df461c1c8cc429bfd0c21aa96e8f3f8c3bd89ce7a752a7a48a930ef406385655d2fd5b8ce46df3aba0697c03357338baee39795adac60d39bd9cb76ee3bdc3f5dffff7f200100000000",
    "000000200790abd6ba5bfb7c03ee8b4f086412542616da84d07b108eb95c5af0f8af7f39b8054769e48aa03787665477dc1b069e4efa6733c23a8e6391752f7f7fd53cb63bdc3f5dffff7f200000000000",
    "000000205b55a2ed9f2109b4bfcf92034721ee9087f5e1d6f2558bbad2865ce8652e497ea3cea5a48d8141139c02a02fe4b297f3856c4b15874fc8bac7e138d296a569f63cdc3f5dffff7f200100000000",
    "00000020bf7b8e2bc4a2c646f7aa6d05e457b101f87f2bd11e901dd24c6a8533f0a65e1b889d79be13adabfb724e9e1fec2625f7b57a082486e9b24e5a4056348f201b753cdc3f5dffff7f200000000000",
    "0000002075d0677cb47c1fe242ea7049cb4dabc1f2736592fa9e315d6734418e1493496f637785b7ab76297a88a6c563cbda5584ee8b03f32b14d55701f26c3209fc021b3cdc3f5dffff7f200200000000",
    "00000020113fb9e8298f1553b75ccb3c7a9e3a8c86a796545a359441af7aa3a8c07d855c480e8b4a5e5e99497951ce51f3b24d96b3e5822c2c08cbbe4d1af1b11e96d4b23cdc3f5dffff7f200000000000",
    "00000020b5ede959f1c035fb680f5833e729f39a283abbd7bb7493741757767b335ef230526502adec73ea9a7378b37259c2d58e7bf5995b996e9cb9326af22ffa17d4ed3cdc3f5dffff7f200100000000",
    "00000020564e94aa75bc57a5ff00d97b2b1ad673ff42a6a5ced3110ad0c8b68ab0088e264ee57cb9a7493f5d57db339249bbfe008a60b370ab86581787d0db24fc593e403cdc3f5dffff7f200200000000",
    "00000020d0bd322a25687101c6f54efdb464bd41b75aba066dc60099362653f84a0e894d06cf3813e5496dc686db35ee1046d5e63009654a4f4faaa73125e048d01118303ddc3f5dffff7f200100000000",
    "00000020ec0678fe9e5da8c415d439cfd23373422e0d68d7c24a6bf46c02e6308fe0f4711a1f29ca87d950e26cccfda040a16b5c096535e78dd9df3a1029a90419e067f33ddc3f5dffff7f200100000000",
    "000000207381a2072d1e7e85dc1bd775c6f8741fb15f82fb6340412f65a0a1b62c28d2691929cf6be8117c46b2e08640443221d813cebfe384e2e9ce3671b2edbaeffc043ddc3f5dffff7f200000000000",
    "00000020fafdad1ba1416f86432af3869302bc7e79117ee5c3688899d350a2829b7c201078e3176de1a5afa8711c378ea2680baf78baffc4f0c3ff346abdfaf7a333b8123ddc3f5dffff7f200000000000",
    "00000020abb19b3146e5cd0342e1d37e9f633b036433bb1f5617d968881ef652165f154655900537b81291567a62ebde98bb8ab14f32b158c0ecfa861a3acbcabd1880f73ddc3f5dffff7f200000000000",
    "00000020a608854bdecf8995a8d3166f193f15b33e87e85fbd6b3220905c3b421898297db0ddf9f07f21963551d764f700454b6f57eab05449e6ddc71ada5d58c89525ea3ddc3f5dffff7f200100000000",
    "00000020882e4c7309554dc12340cce4fefab46c1121bbf4c2388a7bc5284170a7442f39d1dcb690ed54607a2cc8b2c0b7c0c4afd603d5b37036cae17b343cd57429e57d3edc3f5dffff7f200100000000",
    "0000002037e6215a99bfb6a3f28dae40bb754e6812c4e7eea6cb5507f118e50d8df5010900602f6cf9b9b66bb7bf99c3fb748a2e0543c9d45fbb69f16e3c35bc5e9c44ae3edc3f5dffff7f200000000000",
    "00000020667cd4f1bbb49562f9f680430a6bde1a1e598b7f2b67e51a0a7e737d4c0d2d382c2b0ffec75b27ccbffa7cb13a5ed10009b4593c98c587f185633a597d68d17b3edc3f5dffff7f200500000000",
    "000000202262968e4f8bbd4c64b4ed3310c14dda0bc69f2c8af824c55a7c4ce1cff3484482cdd45b99ba92b8a77ed8ba1f8b8989050e4287ff1f4abaa97904617edefceb3edc3f5dffff7f200000000000",
    "00000020599adbba565a8573e884c67ee033e10d9731327390347f0854b3631e4e65744f4f7dde6da928df6dbd6f8b3ae9fdb1d8997c9e9b87fcb8668b08231e1e1b1d753edc3f5dffff7f200100000000",
    "00000020c4d22f1102db94ab2746d8acba73de93c9bda51f8c506ee38deb111664a3d60fd520b4ce003b339b13c20fd06fd0aeef483f32811d332137a508925edea3940c3edc3f5dffff7f200000000000",
    "0000002041824779559111b2ef0d59c5def2d01dff4c6c94afcf85114099af2426ccce3a5ed5d32639d3e081e9216216271b202b062ce6700b8463e6f20e75a03476cdd93fdc3f5dffff7f200100000000",
    "00000020457baa70364759278386b9425b600c79381ef561836754322f6f14b64dac391eb3623a6972fdbf7f789267e1ee73cf3581b15c967fbf882bd3b2575f7693093d3fdc3f5dffff7f200000000000",
    "00000020d578144f562f3c089a931c659c5ede73fd457b570ea68be4ed596720a05e8b7d618a39052cad6379d5c8f9c4e8b82b84e0110c1581910c3fd6b009dc245650b63fdc3f5dffff7f200100000000",
    "00000020c9a7619fc464ae6b70e9e8c28d0a9ce60b82b02fdab27a2eff824548f48aff2ee76955d61f32c46e3927468de40704402c2d05cabdcba0e65f7596f2ab0ee1b33fdc3f5dffff7f200100000000",
    "0000002051c148ff22fa011cda2c6729c78923862fdec9e0013d4086fc9b65cc558f7032301acfddfa26def8deebcbb9abe09899d1935853c0ff2c10d3e214a2de5d3d833fdc3f5dffff7f200000000000",
    "00000020d8a8a7801fc4c0cfeea6160d994837f6900b8f38ec9682b3bf6c2883adfe993721d79d37dfe1498a391fc2a72e28b498904b70a98e989b1cdda19a7842db8e673fdc3f5dffff7f200000000000",
    "000000200018d1eef7fa21c8f6f24eaaa27d75e1b9e014a976677cad4c582e91e3cec25afdd7937495117c0abbd4c3c6bb3066305794b446e8c88e823dae114b1789279e40dc3f5dffff7f200100000000",
    "00000020edc940393936b0b1f856361657b920c45b8f75de1f4e989890f0f26d68d9eb08c62ef7e6771fa46180cc83a8b50e4326228d777049d6b24712cadb03676e653640dc3f5dffff7f200000000000",
    "00000020ea3f1f3f00862f3529fbb267ff87158e0a4591ef66d36b03629133eea26e255d976c31e088b03a4aff9b15f9113a7764fcbc86a29a0e95d0c114bd74b1901ea040dc3f5dffff7f200100000000",
    "00000020caa6ad3ed5198911c9dd8d19587f982e776d2085b353156c95a82f9ffe6a841a2fc5cca1ded18e946203ff80f8a12c0ead407787ca7a5f1471775d2dc248788640dc3f5dffff7f200000000000",
    "0000002040f3c051b48455e4dbbc5d2ee8e4b03165c9e798c5898b82fb4654aca19c94578acab163a47ad0d437bd4cca0f27a3c181931ef0d66d46741fb74760f6f9eb5e40dc3f5dffff7f200300000000",
    "00000020bb0a1537d011680bcd7d51c6bc3d91ec87462b6fc0fad33f371cc86b2316b0477dd0e87ce58c103e5251190859725c0c2a70cebd5703f80cdcbf69c380b4ad2340dc3f5dffff7f200000000000",
    "0000002038f7bfbe77cc905fa1737ca84991ee4cf0db57409a690a919ed31ed7ec56f0040c5f71c8355ac268bf14ec215cccc707da2bf9ab7d18fc9731058794c5caa7fa41dc3f5dffff7f200000000000",
    "00000020d2346725f9bc6aee1981f0a0c4e4594d1a9046d1a4f759919e85146f93f0dc2ef0873d6dfbf0aee6e07571b198805886faa23ee0e389d36880addc8a8d3d4b3941dc3f5dffff7f200100000000",
    "00000020889cf40838bda6ba3662b629edf46a0835ba67c8dad1c4d03fcdb141c6f8a26656e0e67c0e9345ddabddfaab1f2fb791d88a8bfb8cec20fedc8c4340fc0ec4c041dc3f5dffff7f200000000000",
    "00000020406faa9f1b3ba6f27ad532eae977d1446f94fac713495010ff2c53193988d736057ec6530dcc2a8eaac8c0bb2edf2a0fc427d4885411259a90f56c0c8d56c1a941dc3f5dffff7f200000000000",
    "0000002045995f750e636115edade35248351b5b3294a69f81eabf270bcc3c2e441cf62b73fffe3ba7aa7ba3e8aa9f4ac5285ece2e1d810e04452df680371efb87e1fd6a41dc3f5dffff7f200200000000",
    "000000201911ce94bf77845acba60a0ccbcc4673784a22593838e60101a0b16418349b101334b1c74fbbbbe191878a73cc5d2a8e5b106712d37378b7dd6880666f9554c541dc3f5dffff7f200000000000",
    "000000201d896e5f9f8ded05ae4e8bc277f52e9fad1954019e7b867c317c264399de207df02e8766984c4bc2ab1155a4a83c8325aeb2a854276ec40bf0bf94f50b023a0c42dc3f5dffff7f200000000000",
    "00000020bd730420cd37bd45f26f218248fea0232645a2f0a20f846f4c05a3a86b6aea2010c7a7e7741012aa8a3f7772e8bbdab53d77a5d628ba136b45b2ed374640012542dc3f5dffff7f200000000000",
];

pub fn get_test_block_hash(height: usize) -> sha256d::Hash {
    get_test_headers(height, 1).first().unwrap().bitcoin_hash()
}

pub fn get_test_block_index(height: i32) -> BlockIndex {
    let header = get_test_headers(height as usize, 1).pop().unwrap();
    BlockIndex {
        header,
        height,
        next_blockhash: Default::default(),
    }
}

pub fn get_test_headers(start: usize, count: usize) -> Vec<BlockHeader> {
    let mut result: Vec<BlockHeader> = vec![];

    for hex in &HEADER_STRINGS[start..start + count] {
        let bytes = hex_decode(hex).unwrap();
        let header = deserialize(&bytes).unwrap();
        result.push(header);
    }

    result
}

// return initialized chain
pub fn get_chain() -> Chain<OnMemoryChainStore> {
    let mut store = OnMemoryChainStore::new();
    store.initialize(genesis_block(Network::Regtest));
    Chain::new(store)
}

pub struct TwoWayChannel<T> {
    sender: UnboundedSender<T>,
    receiver: UnboundedReceiver<T>,
}

pub fn channel<T>() -> (TwoWayChannel<T>, TwoWayChannel<T>) {
    let (sender_in_here, receiver_in_there) = tokio::sync::mpsc::unbounded_channel::<T>();
    let (sender_in_there, receiver_in_here) = tokio::sync::mpsc::unbounded_channel::<T>();

    let here = TwoWayChannel::new(sender_in_here, receiver_in_here);
    let there = TwoWayChannel::new(sender_in_there, receiver_in_there);

    (here, there)
}

impl<T> TwoWayChannel<T> {
    pub fn new(sender: UnboundedSender<T>, receiver: UnboundedReceiver<T>) -> TwoWayChannel<T> {
        TwoWayChannel { sender, receiver }
    }
}

impl<T> Sink for TwoWayChannel<T> {
    type SinkItem = T;
    type SinkError = Error;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.sender
            .start_send(item)
            .map_err(|e| Self::SinkError::from(e))
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sender
            .poll_complete()
            .map_err(|e| Self::SinkError::from(e))
    }

    fn close(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.sender.close().map_err(|e| Self::SinkError::from(e))
    }
}

impl<T> Stream for TwoWayChannel<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.receiver.poll().map_err(|e| Self::Error::from(e))
    }
}
