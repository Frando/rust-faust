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
process 		= _ : envelop : vbargraph("channel0[unit:dB]", -70, 10) : _;

