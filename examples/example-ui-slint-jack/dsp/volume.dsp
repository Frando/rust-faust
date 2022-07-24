declare name 		"dbmeter";
declare version 	"1.0";
declare author 		"Grame";
declare license 	"BSD";
declare copyright 	"(c)GRAME 2006";

//-------------------------------------------------
// A dB Vumeter
//-------------------------------------------------

import("stdfaust.lib");

envelop			= abs : max(ba.db2linear(-70)) : ba.linear2db : min(10)  : max ~ -(320.0/ma.SR);
vumeter         = _ <: attach(envelop : vbargraph("vumeter", -70, 10)) : _;
volume = vslider("volume", 1, 0, 1, 0.1) : si.smoo;
process 		= _ : vumeter : _ * volume ;

