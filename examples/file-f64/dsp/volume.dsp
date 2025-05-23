declare name        "volume";
declare version     "1.0";
declare author      "Franz Heinzmann";
declare license     "BSD";
declare options     "[osc:on]";

import("stdfaust.lib");

stereo(func) = _,_ : func(_),func(_) : _,_;

volumeM = *(vslider("volume", 0, -70, +4, 0.1) : ba.db2linear : si.smoo);


envelop = abs : max ~ -(1.0/ma.SR) : max(ba.db2linear(-70)) : ba.linear2db;
vumeterM(x) = envelop(x) : vbargraph("level[2][unit:dB][style:dB]", -60, +5);

faderchannel = _,_ : par(i,2,vgroup("channel_%i",volumeM: attach(0,vumeterM))) : _,_;
process = faderchannel;

//a = _*vgroup("channel_1",vslider("volume1", 0, -70, +4, 0.1));
//b = _*vgroup("channel_2",vslider("volume2", 0, -70, +4, 0.1));
//a = _*vslider("volume1", 0, -70, +4, 0.1);
//b = _*vslider("volume2", 0, -70, +4, 0.1);
//faderchannel = _,_ : a,b : _,_;
