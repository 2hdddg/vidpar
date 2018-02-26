use std::io::prelude::*;

use bitreader::BitReader;
use super::*;

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
    transform_8x8_mode_flag: bool,
    pic_scaling_matrix_present_flag: bool,

    second_chroma_qp_index_offset: i8,
}

/*
fn err(text: &str) -> ParserError {
    let unit = ParserUnit::Pps();
    let description = String::from(text);
    let error = ParserUnitError { unit, description };

    ParserError::InvalidStream(error)
}
*/

fn not_impl(text: &str) -> ParserError {
    let unit = ParserUnit::Pps();
    let description = String::from(text);
    let error = ParserUnitError { unit, description };

    ParserError::NotImplemented(error)
}

impl PictureParameterSet {

    pub fn parse<R: Read + Seek>(r: &mut BitReader<R>) ->
            Result<PictureParameterSet> {
        let pic_parameter_set_id = r.ue8()?;
        let seq_parameter_set_id = r.ue8()?;
        let entropy_coding_mode_flag = r.flag()?;
        let bottom_field_pic_order_in_frame_present_flag = r.flag()?;
        let num_slice_groups_minus1 = r.ue8()?;

        if num_slice_groups_minus1 > 0 {
            return Err(not_impl("Slice groups not impl"));
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

        /* Defaults when there is no more rbsp data */
        let mut transform_8x8_mode_flag = false;
        let mut pic_scaling_matrix_present_flag = false;
        let mut second_chroma_qp_index_offset = 0;

        let more_rbsp_data = r.more_rbsp_data()?;
        if more_rbsp_data {
            transform_8x8_mode_flag = r.flag()?;
            pic_scaling_matrix_present_flag = r.flag()?;
            if pic_scaling_matrix_present_flag {
                return Err(not_impl("Scaling matrix not impl"));
            }

            second_chroma_qp_index_offset = r.se8()?;
        }
        r.rbsp_trailing_bits()?;

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
            transform_8x8_mode_flag,
            pic_scaling_matrix_present_flag,
            second_chroma_qp_index_offset,
        })
    }
}
