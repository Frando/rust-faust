declare name        "volumecontrol";
declare version     "1.0";
declare author      "Franz Heinzmann";
declare license     "BSD";
declare options     "[osc:on]";

import("stdfaust.lib");

volume = vslider("volume", 0, 0, 1, 0.1) : si.smoo;

faderchannel = _,_ : _*volume, _*volume : _,_;
process = faderchannel;
