use super::game::{Game, Piece, Side, Square};
use crate::random;
use strum::IntoEnumIterator;

pub type ZobristHash = u64;
pub type ZobristKey = u64;

// Zobrist hashes generated with random_state = 1_804_289_383
pub const ZOBRIST_HASHES: ZobristHashes = ZobristHashes {
    piece_square_hashes: [
        [
            [
                0x41E0_FC38_FD7A_3A74,
                0xF474_89B3_8E60_F819,
                0xADFB_F507_1737_735E,
                0x3183_BE2A_059C_48DF,
                0x7A52_785A_B746_673A,
                0xEF04_3943_9929_81A8,
                0xA807_A3EC_9B2F_D592,
                0x5033_91C4_1BC4_BB1F,
                0xCA40_4675_897A_8696,
                0xC8F1_423C_5D5D_0B59,
                0x446C_6FF9_5E8C_9C6A,
                0x7FB8_A3B7_DDFB_7798,
                0xD5CE_1CAA_B506_57EA,
                0xF3DC_A5E9_4621_A0C3,
                0x3AD7_1FDC_FBB3_989B,
                0x6403_AC3F_8205_68BA,
                0x62B4_1986_8C4F_DCA9,
                0x8FB1_1901_E54A_0303,
                0x4BEC_96A1_3552_E8E9,
                0xA7EF_55C0_7EED_12C7,
                0x4B1E_FC4D_65D6_D0B8,
                0x430A_C05E_6CD4_E869,
                0x58D5_A993_05C9_AA6E,
                0xA8D9_E975_1B97_7097,
                0x713D_431A_11B8_54ED,
                0xA318_357F_DD71_1C3B,
                0x1448_D0D9_F4A7_C424,
                0xF8FE_4A08_9A0C_6C76,
                0xFFEB_6DB3_B569_B656,
                0x7C1A_8E50_C64F_13DF,
                0x83F2_EC04_F1D5_0288,
                0xE330_C9B5_FB91_01A8,
                0x1464_4878_E549_D562,
                0x5A5E_94D6_8216_03EE,
                0xAAF1_3977_4302_AD15,
                0x61C4_50F3_3B3B_4943,
                0x40C2_87F6_5A17_DBFB,
                0x70FB_CAFF_826A_FADF,
                0x8794_2AAB_1CDB_4200,
                0x1BFC_C561_AFBC_D3A0,
                0x4375_4C03_4921_D617,
                0xD24F_47FB_0C7D_A023,
                0x48DF_D980_8389_D00D,
                0xFB10_2D77_E89A_0FF7,
                0x637C_A7CB_297F_394B,
                0x8FFA_BE69_3C79_BAF3,
                0x3CDE_9598_1223_AE4F,
                0x6C11_7FF3_8D82_2C48,
                0x58E9_1080_7CE1_AF40,
                0xB8EC_AF46_A83D_E408,
                0x4180_B12D_8998_FF39,
                0xE497_4E6D_F5DA_70C2,
                0x95D0_E58D_5B56_FE5E,
                0x5C8D_8CE5_1393_23B6,
                0x4D3E_7AF2_12BF_46DA,
                0x7641_97CD_A110_9E70,
                0xB9FF_7EAC_A374_384A,
                0xD1C7_3412_EF49_473C,
                0xB78B_1D14_0DB4_2079,
                0x9C52_241A_3BBF_6AB2,
                0xA91E_8108_6816_4DDA,
                0xF36C_9A34_E2AF_1856,
                0x1D49_8844_3464_6190,
                0xB358_C1B3_ED48_0331,
            ],
            [
                0xC5D6_F62D_F6F4_035D,
                0x1CF1_0B9E_7978_9782,
                0xE424_2DFF_38D5_4A3D,
                0x6246_5453_6782_F612,
                0x2D3A_343B_75A4_BA94,
                0x22BE_6365_2479_525E,
                0x7404_7F86_2BC5_2126,
                0x37A3_6F27_FC28_DB18,
                0x5C54_8DC2_136B_1C81,
                0x5C82_CA32_82D5_ABBF,
                0xF4A8_BB02_2EA5_2D66,
                0x81D5_EA5C_5C6D_4694,
                0xD1F2_8D7D_3170_4127,
                0x800D_B65F_E6C0_17C0,
                0x6920_B948_5F4F_B02C,
                0x1539_2FEE_4ED1_DC16,
                0xFF16_32FB_2FC9_0F60,
                0xF24A_4375_3A6A_2FAE,
                0x936F_0584_7F6D_DDF8,
                0x74EC_172B_A673_A9EE,
                0x369A_2F35_8930_6500,
                0x2533_AC80_6698_0474,
                0x1AF4_149E_D962_E72F,
                0xB218_7CE4_68F1_6BE8,
                0x83E0_4D99_1E28_A2CC,
                0xA0EE_E98F_2FA0_4490,
                0xBCFE_BF42_973F_1E41,
                0xD743_F5F0_72F4_F4C0,
                0xEBB1_CFBB_E48F_657F,
                0x5723_346C_B8A5_59DD,
                0xA9C9_A4B5_E834_DE35,
                0xF8DA_6C17_2658_DBDA,
                0x9729_0750_42A6_DF32,
                0xDB0B_472A_2539_40C7,
                0xFB50_C70B_9A38_74BB,
                0xB6B6_F3BB_D133_A1A3,
                0x10CD_4B25_825C_9ACB,
                0xBF21_7B41_1F3C_1CD2,
                0x59CD_CCE0_077E_BA56,
                0xD47D_7282_BA17_4941,
                0x1813_476F_519C_9D77,
                0x95CA_C4F5_9465_6B63,
                0x669E_FC41_7625_A039,
                0x23BD_E4F4_8748_B094,
                0x4601_D27F_DC18_B1ED,
                0x3486_C072_3C61_261F,
                0xA82E_0993_E868_1351,
                0x5C09_61C1_C838_2D60,
                0x2D1E_44B6_E435_BAF6,
                0x2898_23A0_704F_B7B8,
                0x1F04_802A_8514_C0AC,
                0x25A7_7857_A3C8_5404,
                0x2FE9_D612_8A0D_EE55,
                0x9B35_84E2_A182_B0D3,
                0xBAF4_6EFB_0856_706B,
                0x1A05_B129_623C_1CD7,
                0xDEFC_DE29_EB3D_2315,
                0xF080_7455_8137_ABC9,
                0xB5A5_5698_8632_8F93,
                0x446F_FFA2_13CD_F6A3,
                0x2FC4_A472_18AD_08DB,
                0x3341_B2D2_0486_B082,
                0x7CD2_71B5_4A98_2ECB,
                0xD38C_D0A1_845A_1882,
            ],
            [
                0x09EE_EE3A_8CF1_4DA0,
                0x2C74_28E8_C003_846C,
                0xF4F2_F6F3_3ABB_3228,
                0x9830_15B0_C6CD_1D16,
                0x6DCE_7E27_4F26_DCF6,
                0x05D4_1B55_DF8B_6EB5,
                0x32AA_9FC1_A518_9566,
                0x1C56_BB6B_2CCF_EC27,
                0x259B_7941_AB32_3F82,
                0x83A3_8431_9304_56CF,
                0xB7EF_22CA_9AF6_BEA4,
                0xDBDC_6A05_2417_0E93,
                0x8A7F_AA99_EBC7_F7A5,
                0xBE48_1C3B_2414_31A5,
                0x1B93_8D38_28AC_A7FA,
                0x72C5_A3DB_333B_1AEB,
                0xE098_51D5_0F4E_4E0C,
                0x6B28_E37A_5529_79C5,
                0x7EDB_331B_EEA1_0AC2,
                0x4118_4EF2_72A2_B5D2,
                0x48DF_1394_9AA5_80C3,
                0x695D_9A72_5A2B_07F3,
                0xF8FB_718E_AE14_E72C,
                0x57A0_7673_FDC8_34F9,
                0x7B01_D982_0A47_D00B,
                0xADA4_E378_E1B5_830B,
                0x04FE_A226_046B_C3EA,
                0x8C60_C602_4E10_734F,
                0xEE7F_816C_2698_5F0C,
                0x5445_5768_07D2_41DE,
                0xFFCB_6EC6_13FD_4A28,
                0xAAB4_FF64_A09A_D7F0,
                0xA4D8_CC5C_A8DE_72FD,
                0xF32A_C6C6_B8D6_5D04,
                0x4EC4_97F1_0E09_D5C7,
                0x71C7_BCE0_9C93_4B95,
                0x4B09_9A4A_8FC2_6BC1,
                0xB15C_3E2F_8772_B1F2,
                0xE209_37C8_E12B_2844,
                0x6364_C2CB_74BB_CFF8,
                0x657B_41A6_0FEF_1F14,
                0x6E73_67E6_C1DD_464B,
                0xA3EF_86B3_D0D5_E63F,
                0x1AEC_504F_34B6_4683,
                0x43AF_AB84_B18B_B9CE,
                0x29FD_68E0_3D4C_28FE,
                0x7C93_E034_4C83_6B49,
                0xE8EF_6244_7A28_1C8F,
                0xD6C4_1B33_D174_AE96,
                0x1145_BCAB_DFCF_47FE,
                0xD4B7_CB87_BB4D_F065,
                0xA16E_6D11_919F_C38D,
                0x1C5E_20A7_D756_93B4,
                0x09B2_41EC_9FE0_461C,
                0x970A_9994_7E4C_83F8,
                0x05CC_BF98_AF47_DE41,
                0x4BAF_F6FF_9D85_575C,
                0xE009_FD2C_1055_0B15,
                0x3E2B_E581_70CE_03A8,
                0xD135_21FC_0314_AD9E,
                0xD2E4_B346_4EDC_2DD5,
                0x1982_C9E4_7C0A_B389,
                0xE743_9669_3A02_B92E,
                0x85FB_2990_AAF7_7395,
            ],
            [
                0xCDAE_6574_0328_CA8A,
                0xFAB5_A580_ABCC_A362,
                0xD096_0BD6_33C9_2EA4,
                0x556C_B616_8C7F_DCC6,
                0x8410_BEBC_8BB2_D104,
                0x6D58_0BBC_959B_CE59,
                0x1DD1_EF75_3373_7493,
                0x94B9_1588_C9F9_53DE,
                0x5E1A_78C6_1CF1_5565,
                0xDF73_AFF4_4974_AAF8,
                0x0708_1FE5_BCD9_2E14,
                0xADA5_74C1_FDBF_0419,
                0x3D95_537B_A2E7_B15B,
                0x997D_7A04_948F_6537,
                0x997D_510D_BBE1_50FF,
                0x62C2_4952_A2A5_4DE6,
                0x5578_6D32_25CB_A29D,
                0x1394_4DE3_3B75_BAA4,
                0x4C7A_259B_E5E4_8799,
                0x879A_BA16_AF41_05FE,
                0xDB54_9771_5EB0_C77E,
                0xE1B1_CCF9_B2ED_A9AA,
                0x9620_1FDC_5954_44AB,
                0xB738_18AC_C089_7943,
                0x0E07_D244_C8BB_3EC4,
                0xE8FF_A172_908C_FE01,
                0xD572_A430_788F_2715,
                0x7DC4_D6C3_DC20_30FC,
                0xE1A9_9A82_3A9D_8732,
                0x3026_AA28_7E40_2C27,
                0xBCCC_E22C_B89F_3820,
                0xE0E1_59CE_0658_8041,
                0x1622_B44C_FDE6_5FEF,
                0x399B_5054_A62E_99CB,
                0xF9A2_C1EE_16F8_3123,
                0xFD9E_AC5D_0446_97A0,
                0xF7D4_FC6F_9A46_4CC7,
                0x17DC_D361_9B51_8207,
                0x8115_B7BD_E37D_4895,
                0x80CA_4ADC_5501_296B,
                0x642D_8073_7546_C02B,
                0x8D01_A9A0_5EEA_A3DC,
                0xC374_E67C_485E_6A3A,
                0xC368_9415_7785_EC4C,
                0x7B5A_3D38_CA0E_DAE3,
                0x9E9C_0D9B_77D0_DCCA,
                0xEFFA_3E1F_2E2B_A531,
                0xDDCF_6A2F_CD1C_E2EF,
                0xAA25_7A97_E753_51DE,
                0xE23C_E9FD_8819_1B8A,
                0x0C49_BB36_199A_771B,
                0xCB0F_4240_77AB_BB30,
                0xC513_F309_4C40_EAC1,
                0x0174_9AEA_9DDC_1222,
                0x1E9A_E2F8_25F0_41F2,
                0x959F_0496_E57A_FF7B,
                0xE254_7637_BC99_DD52,
                0x1F7B_8F28_0C45_834D,
                0xC379_6574_83EF_2116,
                0x8028_2B20_4F46_3C3F,
                0xEB7B_30A5_7342_6320,
                0x82FA_6489_B117_BF2C,
                0x7C84_640F_0661_3F35,
                0xA83A_A04D_FC8F_D728,
            ],
            [
                0xC91E_8709_94F2_DC3A,
                0x1074_9E21_0B63_1C91,
                0x9293_8B2F_1DCC_AA4A,
                0x4886_93B4_ACAE_1936,
                0x1A24_6B65_2673_C4A8,
                0x8001_B227_0AAD_5B0B,
                0xC806_3A94_ACFC_3437,
                0xFE98_C22C_7299_D7B9,
                0xD745_24AF_1147_8271,
                0x7D18_746F_DB12_EA11,
                0x6266_9CFA_1F87_D6E5,
                0x4C64_73DA_903F_5777,
                0xBA18_EB53_5689_3569,
                0xBF1D_E0E3_613C_FBEB,
                0x7DB2_3C52_9738_C878,
                0xD9CA_ECA2_53AF_1AF9,
                0x2F3E_8CFD_A2D7_4B32,
                0x5DC8_5416_B7B1_D61D,
                0x5362_4D47_2BC1_33C6,
                0xC616_ED96_FDFF_13DA,
                0x4247_F030_D147_1762,
                0x51DF_1996_5591_91BD,
                0xBD39_D69E_B197_5412,
                0xA83B_3B16_75AA_2FCA,
                0xEE25_E45E_0CE1_B879,
                0x5ABA_9A0B_A1ED_2C8F,
                0x05A5_8993_12FB_C70E,
                0x1B45_614C_F736_8FC3,
                0xBE78_0E75_5B56_A7B6,
                0x71EA_107C_8DC3_8C5B,
                0xA375_526E_F5C9_456C,
                0x9814_627E_037E_9790,
                0x035C_0463_C2D2_A043,
                0x41AB_3F41_747A_BED7,
                0xBBC8_ED09_34C6_0CBE,
                0x6993_F5A9_F14F_66EB,
                0xA269_3DE6_B256_B729,
                0x91BE_FA47_5AD8_E89C,
                0xBA08_795F_1067_6E89,
                0x965A_F9C5_46E3_D942,
                0x69F5_CDBE_6746_A863,
                0xC3A9_AE7A_1CA6_72D1,
                0xD067_CC91_3AB7_6C29,
                0xFB5B_EE1E_A72F_24DB,
                0xAE39_4C13_00C8_9FEC,
                0xA351_52E9_F5ED_5480,
                0x50E4_A881_951F_386C,
                0x866A_4D1C_B3A2_39E2,
                0x53FB_1038_1D67_6A64,
                0x309A_26D0_2C2C_5263,
                0x92E3_41E5_29FB_947D,
                0xAEC7_2125_9EB4_6A97,
                0xF3E3_6866_FC4A_B387,
                0x747C_CD34_03A4_8B80,
                0xBFB8_C5BA_A1AA_7083,
                0xDE62_A149_96B4_886B,
                0xC554_9BE4_AFCD_0546,
                0xF079_B5F3_4A52_7D72,
                0xE58B_0EE4_2E31_352D,
                0x6354_EA5F_D9F4_3843,
                0xA98C_BA28_4ACB_666F,
                0xD01F_AC17_F501_92FA,
                0x4301_9383_C1FF_1F64,
                0x3F4C_91A1_245C_CEE9,
            ],
            [
                0x2752_D599_8C23_291B,
                0xD604_5719_4FE0_EE54,
                0x0285_18C5_93A0_1279,
                0x856B_2AB0_8FCB_8861,
                0xCDF0_448E_1B66_7A36,
                0xD659_B19B_F6A6_98CA,
                0x7854_B6E7_D80D_0DB1,
                0xB74A_C31E_54F6_0F90,
                0x6764_606B_EB36_BE0D,
                0x705B_F929_3676_B4FC,
                0x7EFC_2DD9_DF66_09B5,
                0x0F5C_DA3B_C560_8163,
                0x937D_08CC_0E33_0947,
                0xF768_7E66_EC0C_1403,
                0x4D02_56C5_BBDF_64AB,
                0x1F14_D023_EFFB_1DDB,
                0x2A34_F22A_AD77_20A8,
                0xF77E_BCC2_1570_7D82,
                0x9469_1882_72F1_068B,
                0x9060_81A5_C860_D377,
                0x4FE1_A5C0_9C0C_1E52,
                0xBA6F_1BB4_01DF_5F95,
                0xE304_B698_B14D_B74E,
                0x72C0_B9D7_0848_2D91,
                0xFAA1_52E6_6366_D739,
                0x4529_486F_0188_E146,
                0x69D2_42F8_83E2_7BF1,
                0x6A30_2F5E_7F0E_55F4,
                0x74C4_52FC_35E4_5453,
                0xAA34_323B_F746_BDDF,
                0x9BF9_6555_813D_E28B,
                0xA2B9_1AFD_F085_0E02,
                0x7BB2_8720_044E_009F,
                0x3811_0A1C_10F3_F677,
                0x43DD_DCE3_E947_7852,
                0x9BB1_9CC8_97BF_42F2,
                0xD44E_8064_48A9_4149,
                0x0C18_4E3E_847C_6776,
                0xCF19_1E84_325C_6D01,
                0xFBD3_C7AD_0EF9_9FC9,
                0xD4E7_C02D_C35A_1B34,
                0x35AC_B28D_7F21_7006,
                0xF6A0_74EE_148B_9CE4,
                0x073A_51E1_7922_7D9E,
                0x6E4E_A6F3_13B2_D9E2,
                0xA3C6_1A8E_5389_13AE,
                0x97A5_FDFF_77F9_7CAE,
                0x2196_836C_D9D5_4722,
                0x8D72_F718_CED4_70C1,
                0xFA44_4AC5_EA60_54E1,
                0x3076_63BD_1DEB_E4AB,
                0xE23A_9A97_3692_2784,
                0x8DF2_709F_92F2_1C50,
                0x2778_9274_B4AF_EE4F,
                0x062D_A0C1_67D2_DEA1,
                0x1ED8_AD2A_8F00_C90C,
                0x643D_CF79_9CBE_6A3D,
                0x20CE_ED7C_8F82_4473,
                0x141B_B8BF_3B5D_DEF0,
                0xB3AA_56B3_6C3D_8F29,
                0xA858_3CEE_EC67_A60F,
                0x669B_E796_AF46_CD57,
                0xFD00_597A_514E_566A,
                0x66AD_47F7_0EEC_E61F,
            ],
        ],
        [
            [
                0xB5B6_04BE_005B_43C5,
                0xE76D_762B_7776_2D28,
                0xB773_8565_8F96_F8EE,
                0xDC23_81B7_5E96_D4D3,
                0x281A_AAB4_6B93_DA80,
                0xBB52_9F79_EE74_BA81,
                0xD7D1_B8D3_153A_B745,
                0xBF28_E869_E8E2_15D0,
                0xE84F_A2F4_2DD2_5A24,
                0xCADB_5C01_EDAE_1C55,
                0x3BDF_A822_8F4F_B1EB,
                0xA481_FF33_A9B7_3744,
                0x711C_32B8_6A5F_9110,
                0x2226_A3F8_258A_E5DB,
                0x956A_CA82_81EB_6665,
                0xE8D9_BC7E_C272_C03D,
                0x2F49_E31C_5AB7_3B72,
                0x0C55_95D5_1157_1DBE,
                0x92A8_6C22_BCEC_7915,
                0x294C_7FC8_F87A_888E,
                0x651D_1A52_0439_7DAF,
                0xEC59_B0D3_0E32_6C50,
                0x5972_AF98_0ED8_2BBE,
                0x6DA0_F2F4_5788_1B94,
                0x1016_2AC2_C3DD_5DDB,
                0xFF79_69A0_230E_8177,
                0xD512_DBCC_0017_EA40,
                0x8C19_7AAF_44C2_3CDE,
                0x47AE_D22E_868E_0780,
                0x74F3_7F6E_9397_F114,
                0x4929_53BA_50EE_FA9B,
                0x7F19_6C71_6264_2FFC,
                0xEBFA_BA2E_E49C_B357,
                0xCFD8_7FF1_C85B_B986,
                0x4C7B_264E_EB32_1A19,
                0x5499_CBD6_E45A_4133,
                0xBDE3_1C59_3271_FFFD,
                0xC847_0A90_83FF_1847,
                0x1C94_8C61_4492_E631,
                0xF6A6_87E9_B5BD_8125,
                0xD08D_C54C_96FC_B0A8,
                0xEE63_0BF2_0647_39A3,
                0x5BC1_9FB0_AE00_C196,
                0x0935_EA7D_5CCF_ACC6,
                0x3007_CA50_118E_7C69,
                0xC7C6_0773_E41B_B81A,
                0xED53_CB52_8688_88CD,
                0xA86F_3EA5_DD3A_543F,
                0x1DEC_9E4F_3356_41CA,
                0x51D6_1D82_3A44_2A60,
                0xE18A_2F07_B3CF_D35C,
                0x5D23_45F4_A8CE_9DE0,
                0xC085_9CCE_7ABA_CB89,
                0x4F0E_14CF_03A1_3B52,
                0x8EDD_3B94_8C75_8106,
                0x7A67_C066_9414_3F93,
                0x3783_BE1D_AB5A_E952,
                0xA256_1286_08CB_A4F1,
                0xEF57_B623_6282_2568,
                0xE491_2BDE_8BDE_DBDB,
                0x5FA6_FA74_AA37_67CC,
                0xED40_41C9_9F71_E2BD,
                0xAD51_D2B2_ED9E_3119,
                0x7D50_1A7C_7D5D_595B,
            ],
            [
                0x3EC5_373A_A5A5_2430,
                0x6AC3_1E24_D3E5_D3A1,
                0xFFEB_BCB5_E1A7_BE73,
                0x7613_F51F_B038_5AD9,
                0xB32C_ADCF_DB3A_00A9,
                0x3FBF_13CF_52BE_BC7B,
                0xEF03_0195_2FE4_B23F,
                0x2CEA_FA9C_0DDE_167D,
                0x83C4_DCA8_E680_73A3,
                0xFE13_90D9_6678_22DA,
                0x08AD_B03D_B852_66F7,
                0x8945_8BDD_7658_05B5,
                0xFC02_CC7C_314E_17BA,
                0xD4E3_F5DC_E66D_394B,
                0x45F9_568F_07E2_B3C6,
                0xFE21_CA77_FBCB_4609,
                0x8ECE_961D_2CD5_87A6,
                0x6D1B_B0BE_E3D5_A7B7,
                0x5FF7_23CC_FE8D_540D,
                0xF945_9636_754E_583C,
                0xCAEB_8639_26AA_A1B6,
                0x7562_B236_1473_182C,
                0xF06D_E891_91B7_2705,
                0x7963_1F22_AC26_D2DB,
                0x64E2_1B1B_C84A_C0C1,
                0x6333_C6B4_61A4_D872,
                0xC02A_16F2_AAB5_D473,
                0xD637_8B94_5AF4_1066,
                0x92F4_2169_0260_EA13,
                0x06ED_6B75_8A1F_9987,
                0xA130_E56F_5351_6315,
                0xDFBD_6328_4A85_9198,
                0xE02D_0BC5_4ADF_5306,
                0xD73D_618B_99DA_F02A,
                0xD97D_98E9_C479_FC0C,
                0x148B_E26F_3D54_1086,
                0x326D_C5FE_8ECA_A665,
                0x8E58_E27C_4E2A_CA9C,
                0x3A06_6C7A_2231_546B,
                0x70F0_A25A_196D_A10C,
                0xC31F_DCE4_26BC_FDFF,
                0x6170_4F4E_2C3D_F650,
                0x4454_2A76_8378_30A1,
                0x8715_AA82_A342_DDAA,
                0x8FFA_479C_6B3F_57A8,
                0x09E8_5566_54A8_11AA,
                0x4733_F668_65BC_6D01,
                0x80F7_3876_5A5E_8DB5,
                0xC5E3_02F0_6505_FA05,
                0xA007_E0FD_FEE0_B799,
                0x387A_D2C0_79DE_AE9E,
                0x331E_BDCF_9629_021A,
                0xA71B_3ECE_80DA_349C,
                0x884E_8CA0_1CED_13D5,
                0x10CF_3C56_2A64_77DF,
                0xC529_A556_4179_37A1,
                0x7C3A_8C9F_0A0D_FBC0,
                0xD47C_27AD_AA98_2F1D,
                0x5020_1A94_7488_E740,
                0x9276_94CD_87D1_DC6B,
                0xAD58_A27D_58A6_8D0A,
                0xBAD5_F6AD_80DE_54DE,
                0x8BC8_2545_D52D_4ED3,
                0x15BA_4D72_E7CF_0FD4,
            ],
            [
                0xF8A9_DB73_F611_2DDD,
                0x9273_3216_6530_EA25,
                0x313E_9A23_BB61_F4E8,
                0xE4F5_7F71_004F_348E,
                0x815B_6B73_6188_AC9C,
                0xF7FB_1531_2127_3F52,
                0xB567_21C3_9DD2_7168,
                0xE332_C22C_1523_4C55,
                0xF06A_AE55_5744_AAFB,
                0x3EFD_73FD_BAC5_B536,
                0x0166_9651_BD8A_58DB,
                0x7503_AE1C_1FE3_23FC,
                0xB3E5_E05A_DC48_03E9,
                0xD081_F93E_2D30_4719,
                0x9827_7865_B3A7_BFA2,
                0x7BD4_8CE5_50CA_D4AF,
                0xAE3A_D83A_49AA_5A4D,
                0xCE11_73ED_BC9C_67B1,
                0x78EC_C12B_AF32_D4B0,
                0x39E8_A3D2_255C_BAC8,
                0xF71C_1CD3_8BF5_5AC6,
                0x6DFA_813C_F72C_6354,
                0x874D_8AD2_8F69_4E70,
                0xCCDE_B9C5_18E3_679B,
                0x1A28_2AF7_DB35_E8EA,
                0x6C86_C5C9_54E7_B2FA,
                0x777F_20C6_62A1_EDDF,
                0x9068_1DDA_DF99_028F,
                0x4CA1_B34B_5BE2_2C7F,
                0xD695_7825_6781_B36C,
                0xD03E_D0B5_7F6C_6DC1,
                0x8CBE_A2B0_FB96_6B84,
                0x8533_F136_7840_B2D5,
                0xF966_4770_AD27_69D3,
                0x3CE1_3C97_C8FF_EAA2,
                0x144D_DA1F_450A_C9C3,
                0x417F_3BF0_9428_CD89,
                0x9E69_80A6_5933_DD26,
                0xBAD7_BB74_4780_4D67,
                0xAA26_89AF_60F4_E51A,
                0xFC6A_A571_1A49_99BA,
                0x462E_8BC6_78DC_27DA,
                0xB6C6_A636_39AE_7997,
                0x1470_6C70_DE15_6A2D,
                0xB7A4_2137_0F72_3572,
                0xAED9_867B_9A1C_AAB0,
                0xE196_E463_BFD6_8E0F,
                0x1F2B_E480_A884_B452,
                0x17AD_FF8A_FB17_FA9F,
                0x3150_5FC2_4266_4A0D,
                0x5553_13D2_D19C_94C2,
                0xEFC5_824B_BFA7_8654,
                0x4A86_3E72_1FF2_EC2F,
                0x64ED_17E6_FAB4_67DA,
                0x2A90_4D21_07DC_51E4,
                0x1490_CEC2_93CD_6ECB,
                0xDE64_6925_2403_EF27,
                0xD0C8_FE59_D030_4D0B,
                0xAFF5_EE87_6157_E71B,
                0x28F4_148F_DECF_A798,
                0x3A06_E3EC_19AA_283E,
                0x56C8_E770_CBD3_769A,
                0xA15A_10F7_6110_CCC6,
                0x771F_EADC_559E_458C,
            ],
            [
                0x22DA_26A8_9955_315B,
                0x536D_86CF_2B4F_7314,
                0x0DA8_7DE2_7B22_9B29,
                0x4FCB_FFAC_F40A_5E59,
                0xB7B7_895D_C337_F6E1,
                0x5195_4E94_99D0_2967,
                0xA10E_F4E1_DD56_CBD6,
                0xEE84_2401_AE96_499A,
                0x29AB_9DFE_CE39_31BB,
                0x8C37_3510_BAFB_5447,
                0xD27D_A6B3_B115_D706,
                0x88CA_C501_4D44_80AA,
                0xF021_6237_D685_3B76,
                0xD324_99DC_18B4_A28A,
                0x9785_D8BA_893C_90FC,
                0xD653_C6B0_22F5_F72D,
                0xCC9C_7591_E6F5_06D8,
                0xDABF_0295_74AE_662A,
                0x3417_269A_4467_9DA1,
                0xDBCD_EE04_C14E_672D,
                0x4A5B_8351_A48C_7609,
                0x2D54_BBD2_C441_948A,
                0x9004_6C85_370A_981A,
                0x9F13_1C04_DBDB_C29D,
                0x965C_82B4_9507_48AE,
                0x8367_907B_257F_B523,
                0x8D61_BF8D_5076_9362,
                0x3C2B_18CE_BE54_59D9,
                0xB85D_43FA_262F_6875,
                0xA304_049F_04BB_C24B,
                0xF4C6_3F6A_F707_8712,
                0xF822_441B_2001_8351,
                0x1C5F_0BDB_C47B_B73E,
                0x6D8A_2BC4_2222_9C68,
                0x093F_F41F_2756_95E3,
                0x2331_4632_B1A6_3A49,
                0x6E4C_A730_7757_4A61,
                0x4801_E3B9_34B5_1C7D,
                0xDB11_C16D_CC6A_10A5,
                0x5040_E96D_DBE9_BE1B,
                0x42F2_470D_36B3_484C,
                0x39E9_94C6_A904_536F,
                0xFB81_0B33_6D10_BCDA,
                0x1DCA_5CFC_D986_AF8B,
                0x861D_07CD_8D65_15B8,
                0x6895_EF18_6082_D6E7,
                0xFA6D_D6A3_8457_89FB,
                0x0F0E_F121_87F1_A0CD,
                0xB2D2_2F7E_D4B6_4A7C,
                0xFBE2_D915_0201_EAA9,
                0x6198_B644_6CAA_656B,
                0x6BAC_C9DA_A5B8_031E,
                0xF67B_921C_DA30_216A,
                0xA30F_8D8C_AEC7_332E,
                0x3B26_4284_820E_934B,
                0xA815_B25D_4580_A6FA,
                0xB56B_95C6_A7D4_7E0B,
                0x9266_E7A6_1254_0C33,
                0xCF35_B204_8DDB_40D4,
                0x54BC_EBE6_15C2_C070,
                0xCBEE_3D62_34C9_ADBE,
                0x1D38_BF5C_942F_BE1C,
                0x6848_E136_2E0A_D65A,
                0xD556_C2FD_0347_6FB6,
            ],
            [
                0xAF66_C0E9_1D57_435A,
                0x1FF1_354F_F877_9362,
                0xB2DC_3F16_0815_F61A,
                0x7593_C0E7_EFC1_AAA2,
                0xE05D_33B0_6F1E_328E,
                0x497A_029D_E939_1F24,
                0x13E7_B1CE_DA10_E3F2,
                0x6544_F0E3_5A44_DDC7,
                0x0F77_8418_5DC0_819A,
                0x6F16_CED2_B6FE_0B64,
                0x3A86_D5E4_31F7_EDBE,
                0x89AE_234B_0EC4_C1BB,
                0xCFB2_0CD6_8B53_1491,
                0x1818_5BB9_A647_B4B2,
                0xD05B_BC76_BAB2_FEFF,
                0xFB9F_8EE6_592D_3BA0,
                0xD715_5204_F025_8E98,
                0x9B7D_A44A_6FA9_B28B,
                0x9ACF_8D25_D22E_B215,
                0x4FFA_ED5F_28F4_55EE,
                0xBC81_69CF_A643_28F5,
                0x2671_AA08_4F4D_E45D,
                0x3AB4_C0B0_BDEE_07D5,
                0x5C2D_CE08_A08F_1500,
                0xBFE5_6F80_5524_8244,
                0x2B06_7300_B617_BB1B,
                0x0061_BE74_543C_8715,
                0xD5FB_C1D4_3B0C_6CA2,
                0xFCD4_6921_3082_257B,
                0xC033_A813_8A54_693F,
                0x5839_6DD8_0A3F_7978,
                0x7EC4_5C4C_685E_4858,
                0x2CEB_8A92_EE5B_F74B,
                0x19C8_AFA7_94F6_3AA5,
                0xCBFE_4E2B_1806_01BB,
                0x9709_8973_CEA0_E9F0,
                0xC3FD_65A7_B36B_43AE,
                0x11EF_B57C_7996_F1E4,
                0xA06E_5D11_CDEC_C783,
                0x0B21_99E7_4204_F3F6,
                0xE0B3_5F0E_F64B_C3D3,
                0x5CCA_BFD3_178F_CB1E,
                0xF96B_474D_E9AE_1E51,
                0xB3C4_598E_13E9_EDE4,
                0x757B_E8E9_EB98_3C9F,
                0x9C59_8429_3819_5225,
                0x1B00_64A2_C2FD_E2B3,
                0x5028_096F_DC4F_2BF5,
                0x7777_6804_1110_63D2,
                0xBB95_487A_CCA8_020F,
                0x2BA6_9B1A_4D1E_FED2,
                0xD3F0_5DD1_91CF_B57C,
                0x26FD_29A7_B937_E3C8,
                0x111A_89B6_8437_7930,
                0x9741_42FF_7648_D032,
                0x82D9_1213_80E1_7AF4,
                0xAA1C_A4D9_7A4C_A3FD,
                0xCD23_B12F_993B_BD61,
                0x1634_05CB_4BD9_DB7B,
                0x7F12_9A43_3E93_04C8,
                0x77F9_452F_5046_DC2F,
                0x9D87_1BBF_FF7D_C07E,
                0x1D5B_1282_F5F7_A61C,
                0x1A88_99A7_A539_0342,
            ],
            [
                0xEF43_2B3D_07F3_E1C9,
                0x942D_3443_659A_9E41,
                0xA5E4_C5F9_7A7A_CB13,
                0xA6E8_590D_02E4_716F,
                0x3D24_ACA4_2FE4_891E,
                0x28B5_BCF7_DE7D_C726,
                0x3E1A_84EB_D1E7_F1CA,
                0x1FB4_71D2_5F09_7603,
                0x1BC5_530D_4DFD_F72A,
                0x9182_48F6_53F5_A2CE,
                0xA86C_2898_4B60_6BA4,
                0xF755_723F_4C92_32FF,
                0xC7DB_EA29_9A21_7E66,
                0x095A_15C9_F996_FE49,
                0x37B0_1BAB_EBDE_A168,
                0x2EB4_0213_1DAD_DBB0,
                0xA876_A815_4230_C1DA,
                0xF234_E574_FDBA_4485,
                0x1711_0699_163B_9EF1,
                0x7719_EE0C_75D8_BFBB,
                0x2D1E_F912_3B96_57BA,
                0xEB1A_3284_B9D5_9A1C,
                0x256B_BEAA_345D_3BDB,
                0xF417_B1F8_F42E_1F76,
                0x02FE_9A90_537F_67D3,
                0xD036_7142_DB17_8C05,
                0xF508_F867_8990_D631,
                0x9006_2C87_BB8C_E1D0,
                0xEF8D_2796_43DC_888F,
                0x61AC_0C0F_A50F_6D34,
                0xE5B4_97F3_2769_A68B,
                0x0046_24D9_C30E_9FC0,
                0x0DE7_38EF_6F88_607C,
                0x2650_5AA6_FB93_4EBB,
                0x7357_8362_F377_DD29,
                0x0012_466B_31FE_F96B,
                0x14B4_AF64_0F3F_68DE,
                0x44EB_DE9A_F6F4_A5FA,
                0x52A0_298D_9A72_5AFB,
                0x28A2_5884_908F_BCEF,
                0x3424_2CBB_9432_6835,
                0x5055_0F9E_0C94_EBFB,
                0x5E1E_3167_4DD8_C333,
                0x6B75_4720_B505_C778,
                0xF137_D7B8_D2A0_37D2,
                0x99B1_1C7F_C348_D9D3,
                0x382C_FB41_5729_995A,
                0x0F0E_39DF_2CCC_8F9B,
                0xBD93_3A86_89CC_E892,
                0x8F30_2DA7_8DB2_C1E3,
                0x8411_945B_A81B_E46E,
                0x03DC_4D50_82F3_6BA6,
                0x9067_44C5_B12F_08DD,
                0xCA63_03B4_C472_EA1B,
                0xE347_C1FD_BDB0_DADA,
                0x2E3A_F0D5_E0F4_5D68,
                0xB6A0_94D2_60D9_6846,
                0xDA60_E407_51CF_6926,
                0xA85D_EBBF_9048_F350,
                0x4E9E_D463_EDAD_FF2F,
                0x716A_C5E9_0251_1274,
                0x28CC_B295_BEA6_E5D0,
                0x0B44_5C21_AB6B_B599,
                0xF41D_BB29_AA10_BCE2,
            ],
        ],
    ],
    en_passant_square_hashes: [
        0x6AC9_6FBF_508C_6B50,
        0xDCFF_6F9B_61C1_E1BA,
        0x8A00_C4D1_7CD4_55C7,
        0x8B88_C53A_82A3_0DFD,
        0x3F08_193E_0367_913F,
        0x8DE9_C12C_DB1E_ED21,
        0x52A4_3F47_BD44_5003,
        0xDECD_36D4_ED6B_A204,
        0x2B40_67D2_8330_25C3,
        0xF7F1_D29C_6217_FB62,
        0x64F0_7273_27D0_4097,
        0xCA29_DE31_A51F_01BF,
        0x3825_2A32_4DBC_5399,
        0x85B1_A2A5_5F15_53A2,
        0x9C74_82CB_DF2F_5579,
        0xD8F3_58FC_51F7_1E60,
        0x147B_4E16_2307_978B,
        0x764E_04E5_9CD8_CC4E,
        0x7F19_C075_1ED7_7A09,
        0xEBE1_71A7_C71E_0F53,
        0xCC4F_61F5_0BA0_87E0,
        0x0F11_995C_95A8_505E,
        0x7874_48F7_430B_35DF,
        0xF93F_E0AA_46D0_A2DA,
        0x09CE_58B2_082D_7413,
        0xDEC9_6CE7_A619_13AE,
        0xA697_78E5_9A2E_01C1,
        0xD300_1E9F_75BA_E8F6,
        0x7B59_5E37_4946_A8C4,
        0x5A47_EC5A_4632_80DD,
        0x792B_5881_602F_B666,
        0x71F3_A76D_1FB2_DBA4,
        0xAAF3_15CD_DA1E_E582,
        0x8598_3C5A_DD40_1DF9,
        0xA0A8_1692_B9E9_6596,
        0x06D0_F142_BCF9_27E9,
        0x13EC_7D6E_9C2C_57B6,
        0x0CC1_6969_5C51_C02D,
        0x2A83_07A5_3877_E614,
        0xE1AB_07D4_4F3C_25E2,
        0xCA71_F151_83E5_F971,
        0x8D48_7E9B_E305_3E8A,
        0xB6AB_2A89_68AE_12DF,
        0x8B43_F76F_A4A7_F827,
        0x404E_2F40_17A6_859D,
        0x3966_1BB4_26D6_D933,
        0x70FA_D8D3_80D7_39FA,
        0x67BF_9EE0_740D_E124,
        0x86D3_5A67_A1A5_1425,
        0x6130_DA78_7890_6283,
        0x5869_4AFB_6FD1_6D13,
        0x7535_8C98_783B_614A,
        0x1637_F448_9956_80A3,
        0x8ABC_F65C_DFED_6843,
        0xF689_F052_3657_D9C5,
        0x1556_0A9D_0595_4922,
        0xF7DD_AB91_DC75_94A6,
        0x1072_1A5C_B135_FEA3,
        0xA8A3_9608_FD24_7806,
        0x9629_2319_DB72_1EB9,
        0x2318_21B2_6BCE_90CC,
        0x15D6_4E45_3383_FFF1,
        0x5F44_CE53_389D_0347,
        0x2257_4E06_318E_F86D,
    ],
    castling_hashes: [
        0xCA9A_34CF_DC77_63F8,
        0x74AA_E1A5_487C_E2B9,
        0x0C18_E4F0_4ADC_0B2B,
        0x7A06_6350_F617_7FCF,
        0x3FB1_376E_44D1_02A2,
        0x38A9_E30F_3AEA_62FD,
        0x5D8A_BA50_53E4_2FD5,
        0xD816_6567_012D_C504,
        0x2052_C98F_874E_C5CE,
        0x0715_1F39_67BF_65FC,
        0x7F16_8100_565C_871B,
        0xC5D2_8E66_8459_6B4D,
        0x7AE4_3D8C_2F57_63E8,
        0xC0D0_D41A_0FC2_9E89,
        0xAF2C_553D_5D17_928E,
        0xA7D3_269F_109A_1670,
    ],
    side_hash: 0x1897_8258_A558_3E6B,
};

const CASTLING_RIGHTS_PERMUTATIONS: usize = 16;

pub struct ZobristHashes {
    piece_square_hashes: [[[ZobristHash; 64]; 6]; 2],
    en_passant_square_hashes: [ZobristHash; 64],
    castling_hashes: [ZobristHash; CASTLING_RIGHTS_PERMUTATIONS],
    side_hash: ZobristHash,
}

impl ZobristHashes {
    fn _initialise(random_state: &mut u32) -> Self {
        let mut piece_square_hashes = [[[0; 64]; 6]; 2];

        for side in Side::iter() {
            for piece in Piece::iter() {
                for square in Square::iter() {
                    piece_square_hashes[side as usize][piece as usize][square as usize] =
                        random::_generate_random_u64(random_state);
                }
            }
        }

        let mut en_passant_square_hashes = [0; 64];

        for square in Square::iter() {
            en_passant_square_hashes[square as usize] = random::_generate_random_u64(random_state);
        }

        let mut castling_hashes = [0; CASTLING_RIGHTS_PERMUTATIONS];

        for castling_key in &mut castling_hashes {
            *castling_key = random::_generate_random_u64(random_state);
        }

        let side_hash = random::_generate_random_u64(random_state);

        Self {
            piece_square_hashes,
            en_passant_square_hashes,
            castling_hashes,
            side_hash,
        }
    }

    pub fn generate_key(&self, game: &Game) -> ZobristKey {
        let mut key = 0;

        for (mut bitboard, piece, side) in game.piece_bitboards() {
            while let Some(square) = bitboard.get_lsb_square() {
                key ^= self.piece_square_hashes[side as usize][piece as usize][square as usize];

                bitboard.pop_bit(square);
            }
        }

        if let Some(square) = game.en_passant_square() {
            key ^= self.en_passant_square_hashes[square as usize];
        }

        key ^= self.castling_hashes[game.castling_rights_value() as usize];
        key ^= self.side_hash * game.side_to_move() as ZobristHash;

        key
    }

    pub fn piece_square_hash(&self, piece: Piece, side: Side, square: Square) -> ZobristHash {
        self.piece_square_hashes[side as usize][piece as usize][square as usize]
    }

    pub fn en_passant_square_hash(&self, square: Square) -> ZobristHash {
        self.en_passant_square_hashes[square as usize]
    }

    pub fn castling_hash(&self, castling_type: u8) -> ZobristHash {
        self.castling_hashes[castling_type as usize]
    }

    pub fn side_hash(&self) -> ZobristHash {
        self.side_hash
    }
}
