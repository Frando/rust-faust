declare name "mixer-4ch";
declare version "0.0";
declare author "obsoleszenz";
declare description "A 4 channel dj mixer, with 2 stereo send/returns, a main, booth and cue output";
declare options "[midi:on]";

import("stdfaust.lib");

param_volumes = par(i, 6, hslider("volume%i", 1, 0, 1, 0.1) : si.smoo);
param_gains = par(i, 6, hslider("gain%i", 0.5, 0, 1, 0.1) : si.smoo);
params_cue_toggles = par(i, 4, button("cue_toggle%i"));


input_channel = 
	_,_, // ch1
	_,_, // ch2
	_,_, // ch3
	_,_; // ch4

input_return_fx1 = _,_;
input_return_fx2 = _,_;


output_main = _,_;
output_booth = _,_;
output_cue = _,_;
output_send_fx1 = _,_;
output_send_fx2 = _,_;



stage_add_outputs = 
	input_channel,
	input_return_fx1,
	input_return_fx2
		:
			input_channel,
			input_return_fx1,
			input_return_fx2,
			0,0, // output_main
			0,0, // output_booth
			0,0, // output_cue
			0,0, // output_send_fx1
			0,0;  // output_send_fx2


stage_remove_inputs = 
	input_channel,
	input_return_fx1,
	input_return_fx2,
	output_main,
	output_booth,
	output_cue,
	output_send_fx1,
	output_send_fx2
		:
		  si.block(8), // terminate input_channel
		  si.block(2), // terminate input_return_fx1
		  si.block(2), // terminate input_return_fx2
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2;


stereo_gain(gain) = _,_ : par(i, 2, _ * gain): _,_;

stage_gain_and_volume_for_channels_and_fx(param_gains, param_volumes) =
	input_channel,
	input_return_fx1,
	input_return_fx2,
	output_main,
	output_booth,
	output_cue,
	output_send_fx1,
	output_send_fx2
		:
			par(
				i,
				6, 
				(
					_,_
					:
						stereo_gain(ba.selector(i, 6, param_gains) * 2)
					:
						stereo_gain(ba.selector(i, 6, param_volumes))
					:
					_,_
				)
			),
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
with {
};


stage_main_mixdown = 
	input_channel,
	input_return_fx1,
	input_return_fx2,
	output_main,
	output_booth,
	output_cue,
	output_send_fx1,
	output_send_fx2
		:
			(input_channel <: input_channel, input_channel),
			input_return_fx1,
			input_return_fx2,
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		:
			(input_channel, input_channel : input_channel, (input_channel :> si.bus(2))),
			input_return_fx1,
			input_return_fx2,
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		: 
			input_channel,
			output_main,
			input_return_fx1,
			input_return_fx2,
			si.block(2), // terminate old main output
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		:
			(
				input_channel,
				output_main,
				input_return_fx1,
				input_return_fx2
					: route(
							14,
							14,
							(1,1),(2,2),(3,3),(4,4),(5,5),(6,6),(7,7),(8,8), // input_channel
							(11,9),(12,10),                                  // input_return_fx1
							(13,11), (14, 12),                               // input_return_fx2
							(9, 13), (10, 14)                                // output main
						)
			),
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		;

stage_cue_mixdown(params_cue_toggles) =
	input_channel,
	input_return_fx1,
	input_return_fx2,
	output_main,
	output_booth,
	output_cue,
	output_send_fx1,
	output_send_fx2
		:
			(input_channel <: input_channel, input_channel),
			input_return_fx1,
			input_return_fx2,
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		: 
			input_channel,
			cue_toggle(params_cue_toggles),
			input_return_fx1,
			input_return_fx2,
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		: 
			input_channel,
			(input_channel :> _,_),
			input_return_fx1,
			input_return_fx2,
			output_main,
			output_booth,
			output_cue,
			output_send_fx1,
			output_send_fx2
		: 
			input_channel,
			(
				_,_, // mixed down cue
				input_return_fx1,
				input_return_fx2,
				output_main,
				output_booth,
				output_cue
					:
						_,_, // mixed down cue
						input_return_fx1,
						input_return_fx2,
						output_main,
						output_booth,
						!,!
					:
						route(
							10,
							10, 
							(1, 9), (2, 10), // cue
							(3, 1), (4, 2),  // return fx1 
							(5, 3), (6, 4),  // return fx2
							(7, 5), (8, 6),  // main
							(9, 7), (10, 8)  // booth
						)
				),
				output_send_fx1,
				output_send_fx2
			
with {
};

cue_toggle(params_cue_toggles) =
	input_channel
		:
			par(i, 4, stereo_mute(ba.selector(i, 4, params_cue_toggles)))
with {
	stereo_mute(bpc) = _,_ : ba.bypass2(bpc, (_*0,_*0)) : _,_;
};


process = 
	input_channel,
	input_return_fx1,
	input_return_fx2
		: stage_add_outputs
		: stage_gain_and_volume_for_channels_and_fx(param_gains, param_volumes)
		: stage_main_mixdown
		: stage_cue_mixdown(params_cue_toggles)
		: stage_remove_inputs
		;


//process = stage_cue_mixdown(params_cue_toggles);

