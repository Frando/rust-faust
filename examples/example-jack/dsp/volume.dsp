declare name        "volumecontrol";
declare version     "1.0";
declare author      "Franz Heinzmann";
declare license     "BSD";
declare options     "[osc:on]";

import("stdfaust.lib");

stereo(func) = _,_ : func(_),func(_) : _,_;

volumeM = *(vslider("volume", 0, -70, +4, 0.1) : ba.db2linear : si.smoo);
volume = stereo(volumeM);

envelop = abs : max ~ -(1.0/ma.SR) : max(ba.db2linear(-70)) : ba.linear2db;
vumeterM(x) = envelop(x) : vbargraph("level[2][unit:dB][style:dB]", -60, +5);
vumeterS(a,b) = a,b <: _,_,_,_ : 
  (a, b, attach(0,vumeterM((a+b)/2)), 0) :>
  _,_;
vumeter = _,_ : vumeterS(_,_);

faderchannel = _,_ : volume : vumeter : _,_;
process = faderchannel;
