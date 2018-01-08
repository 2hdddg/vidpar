use std::io::prelude::*;

use bitreader::BitReader;

#[derive(Debug)]
pub struct SequenceScalingMatrixEntry {
    seq_scaling_list_present_flag: bool,
    delta_scale: i32,
}

#[derive(Debug)]
pub struct VideoUsabilityInformation {
    aspect_ratio_info_present_flag: bool,
}

#[derive(Debug)]
pub struct SequenceParameterSet {
    profile_idc: u8,
    constraint_set0_flag: bool,
    constraint_set1_flag: bool,
    constraint_set2_flag: bool,
    constraint_set3_flag: bool,
    constraint_set4_flag: bool,
    constraint_set5_flag: bool,
    level_idc: u8,
    seq_parameter_set_id: u8,
    chroma_format_idc: u8,
    separate_colour_plane_flag: bool,
    bit_depth_luma_minus8: u8,
    bit_depth_chroma_minus8: u8,
    qpprime_y_zero_transform_bypass_flag: bool,
    seq_scaling_matrix_present_flag: bool,
    //scaling_list: Vec<SequenceScalingMatrixEntry>,
    log2_max_frame_num_minus4: u32,
    pic_order_cnt_type: u8,
    log2_max_pic_order_cnt_lsb_minus4: u8,
    delta_pic_order_always_zero_flag: bool,
    offset_for_non_ref_pic: i64,
    offset_for_top_to_bottom_field: i64,
    num_ref_frames_in_pic_order_cnt_cycle: u8,
    offset_for_ref_frame: Vec<i64>,
    max_num_ref_frames: u8,
    gaps_in_frame_num_value_allowed_flag: bool,
    pic_width_in_mbs_minus1: u32,
    pic_height_in_map_units_minus1: u32,
    frame_mbs_only_flag: bool,
    mb_adaptive_frame_field_flag: bool,
    direct_8x8_inference_flag: bool,
    frame_cropping_flag: bool,
    frame_crop_left_offset: u32,
    frame_crop_right_offset: u32,
    frame_crop_top_offset: u32,
    frame_crop_bottom_offset: u32,
    vui_parameters_present_flag: bool,
    vui_parameters: Option<VideoUsabilityInformation>,
}

impl SequenceParameterSet {
    pub fn parse<R: Read>(r: &mut BitReader<R>) ->
                          Result<SequenceParameterSet, &'static str> {
        let profile_idc = r.b()?;
        let constraint_set0_flag = r.flag()?;
        let constraint_set1_flag = r.flag()?;
        let constraint_set2_flag = r.flag()?;
        let constraint_set3_flag = r.flag()?;
        let constraint_set4_flag = r.flag()?;
        let constraint_set5_flag = r.flag()?;

        let reserved_zero_2bit = r.u64(2)?;
        if reserved_zero_2bit != 0 {
            return Err("SPS: Reserved zero 2 bit is not zero");
        }

        let level_idc = r.u8(8)?;
        let seq_parameter_set_id = r.ue8()?;
        if seq_parameter_set_id > 31 {
            return Err("SPS: seq_parameter_set_id too large");
        }

        /* 1 => 4:2:0 */
        let mut chroma_format_idc = 1;
        let mut separate_colour_plane_flag = false;
        let mut bit_depth_luma_minus8 = 0;
        let mut bit_depth_chroma_minus8 = 0;
        let mut qpprime_y_zero_transform_bypass_flag = false;
        let mut seq_scaling_matrix_present_flag = false;
        let mut log2_max_pic_order_cnt_lsb_minus4 = 0;
        let mut delta_pic_order_always_zero_flag = false;
        let mut offset_for_non_ref_pic = 0;
        let mut offset_for_top_to_bottom_field = 0;
        let mut num_ref_frames_in_pic_order_cnt_cycle = 0;
        let mut offset_for_ref_frame: Vec<i64> = Vec::new();

        match profile_idc {
            100 | 110 | 122 | 244 |  44 |
             83 |  86 | 118 | 128 | 138 |
            139 | 134 | 135 => {
                chroma_format_idc = r.ue8()?;
                if chroma_format_idc > 3 {
                    return Err("SPS: chroma_format_idc too large");
                }
                if chroma_format_idc == 3 {
                    separate_colour_plane_flag = r.flag()?;
                }
                bit_depth_luma_minus8 = r.ue8()?;
                if bit_depth_chroma_minus8 > 6 {
                    return Err("SPS: bit_depth_luma_minus8 too large");
                }
                bit_depth_chroma_minus8 = r.ue8()?;
                if bit_depth_chroma_minus8 > 6 {
                    return Err("SPS: bit_depth_chroma_minus8 too large");
                }
                qpprime_y_zero_transform_bypass_flag = r.flag()?;
                seq_scaling_matrix_present_flag = r.flag()?;

                if seq_scaling_matrix_present_flag {
                    return Err("SPS: scaling matrix not implemented");
                }
            },
            _ => {},
        };

        let log2_max_frame_num_minus4 = r.ue32()?;
        if log2_max_frame_num_minus4 > 12 {
            return Err("SPS: log2_max_frame_num_minus4 larger than 12");
        }

        let pic_order_cnt_type = r.ue8()?;
        match pic_order_cnt_type {
            0 => {
                log2_max_pic_order_cnt_lsb_minus4 = r.ue8()?;
            },
            1 => {
                delta_pic_order_always_zero_flag = r.flag()?;
                offset_for_non_ref_pic = r.se64()?;
                offset_for_top_to_bottom_field = r.se64()?;
                num_ref_frames_in_pic_order_cnt_cycle = r.ue8()?;
                /* Read offsets */
                for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                    offset_for_ref_frame.push(r.se64()?);
                }
            },
            2 => {},
            _ => return Err("SPS: pic_order_cnt_type larger than 2"),
        }

        /* Valid range 0 to MaxDpbFrames */
        /* MaxDpbFrames is equal to:
         * min(mvScaleFactor * MaxDpbMbs / (PicWidthInMbs * FrameHeightInMbs),
               max(1, ceil(log2(NumViews))) * 16)
         */
        let max_num_ref_frames = r.ue8()?;
        let gaps_in_frame_num_value_allowed_flag = r.flag()?;
        let pic_width_in_mbs_minus1 = r.ue32()?;
        let pic_height_in_map_units_minus1 = r.ue32()?;
        let frame_mbs_only_flag = r.flag()?;
        let mut mb_adaptive_frame_field_flag = false;
        if frame_mbs_only_flag {
            mb_adaptive_frame_field_flag = r.flag()?;
        }
        let direct_8x8_inference_flag = r.flag()?;
        let frame_cropping_flag = r.flag()?;

        let mut frame_crop_left_offset = 0;
        let mut frame_crop_right_offset = 0;
        let mut frame_crop_top_offset = 0;
        let mut frame_crop_bottom_offset = 0;
        if frame_cropping_flag {
            frame_crop_left_offset = r.ue32()?;
            frame_crop_right_offset = r.ue32()?;
            frame_crop_top_offset = r.ue32()?;
            frame_crop_bottom_offset = r.ue32()?;
        }
        let vui_parameters_present_flag = r.flag()?;

        let vui_parameters = None;
        if vui_parameters_present_flag {
            return Err("SPS: vui parameters not implemented");
        }

        /* Rules */
        if !frame_mbs_only_flag &&
           !direct_8x8_inference_flag {
            return Err("SPS: invalid");
        }

        Ok(SequenceParameterSet {
            profile_idc,
            constraint_set0_flag,
            constraint_set1_flag,
            constraint_set2_flag,
            constraint_set3_flag,
            constraint_set4_flag,
            constraint_set5_flag,
            level_idc,
            seq_parameter_set_id,
            chroma_format_idc,
            separate_colour_plane_flag,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag,
            seq_scaling_matrix_present_flag,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4,
            delta_pic_order_always_zero_flag,
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frame,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed_flag,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            frame_mbs_only_flag,
            mb_adaptive_frame_field_flag,
            direct_8x8_inference_flag,
            frame_cropping_flag,
            frame_crop_left_offset,
            frame_crop_right_offset,
            frame_crop_top_offset,
            frame_crop_bottom_offset,
            vui_parameters_present_flag,
            vui_parameters,
        })
    }
}
