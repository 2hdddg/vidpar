use std::io::prelude::*;

use bitreader::BitReader;

#[derive(Debug)]
pub struct PictureParameterSet {
    pic_parameter_set_id: u8,
    seq_parameter_set_id: u8,
    entropy_coding_mode_flag: bool,
    bottom_field_pic_order_in_frame_present_flag: bool,
    num_slice_groups_minus1: u8,
/*
    slice_group_map_type: u8,
*/
    num_ref_idx_l0_default_active_minus1: u8,
    num_ref_idx_l1_default_active_minus1: u8,
    weighted_pred_flag: bool,
    weighted_bipred_idc: u8,
    pic_init_qp_minus26: i8,
    pic_init_qs_minus26: i8,
    chroma_qp_index_offset: i8,
    deblocking_filter_control_present_flag: bool,
    constrained_intra_pred_flag: bool,
    redundant_pic_cnt_present_flag: bool,
}

impl PictureParameterSet {

    pub fn parse<R: Read + Seek>(r: &mut BitReader<R>) ->
            Result<PictureParameterSet, &'static str> {
        let pic_parameter_set_id = r.ue8()?;
        let seq_parameter_set_id = r.ue8()?;
        let entropy_coding_mode_flag = r.flag()?;
        let bottom_field_pic_order_in_frame_present_flag = r.flag()?;
        let num_slice_groups_minus1 = r.ue8()?;

        if num_slice_groups_minus1 > 0 {
            return Err("PPS: Slice groups not impl");
        }

        /* Range 0 - 31 */
        let num_ref_idx_l0_default_active_minus1 = r.ue8()?;
        /* Range 0 - 31 */
        let num_ref_idx_l1_default_active_minus1 = r.ue8()?;

        let weighted_pred_flag = r.flag()?;
        let weighted_bipred_idc = r.u8(2)?;
        /* -26 to 25 inclusive */
        let pic_init_qp_minus26 = r.se8()?;
        /* -26 to 25 inclusive */
        let pic_init_qs_minus26 = r.se8()?;
        /* -12 to 12 inclusive */
        let chroma_qp_index_offset = r.se8()?;
        let deblocking_filter_control_present_flag = r.flag()?;
        let constrained_intra_pred_flag = r.flag()?;
        let redundant_pic_cnt_present_flag = r.flag()?;

        let more_rbsp_data = r.more_rbsp_data()?;
        if more_rbsp_data {
        }

        Ok(PictureParameterSet {
            pic_parameter_set_id,
            seq_parameter_set_id,
            entropy_coding_mode_flag,
            bottom_field_pic_order_in_frame_present_flag,
            num_slice_groups_minus1,
            num_ref_idx_l0_default_active_minus1,
            num_ref_idx_l1_default_active_minus1,
            weighted_pred_flag,
            weighted_bipred_idc,
            pic_init_qp_minus26,
            pic_init_qs_minus26,
            chroma_qp_index_offset,
            deblocking_filter_control_present_flag,
            constrained_intra_pred_flag,
            redundant_pic_cnt_present_flag,
        })
    }
}
