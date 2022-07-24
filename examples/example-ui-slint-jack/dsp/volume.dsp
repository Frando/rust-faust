declare name 		"dbmeter";
declare version 	"1.0";
declare author 		"Grame";
declare license 	"BSD";
declare copyright 	"(c)GRAME 2006";

//-------------------------------------------------
// A dB Vumeter
//-------------------------------------------------

import("stdfaust.lib");


volume = hslider("volume", 0, 0, 1, 0.01) : si.smoo;
vumeter(x) = attach(x, envelop(x) : hbargraph("[2][unit:dB]", -70, +5));
envelop = abs : max ~ -(1.0/ma.SR) : max(ba.db2linear(-70)) : ba.linear2db;
process = _ : _ * volume : vumeter : _;

