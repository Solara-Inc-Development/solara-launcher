<script setup lang="ts">
import {useId} from 'vue';
const id=useId().replace(/:/g,'');
</script>
<template>
<div class="portal-art" role="img" aria-label="A glowing Solara cube floating inside a light ring">
  <div class="portal-grid"></div><div class="portal-haze"></div>
  <div class="portal-scene">
    <div class="portal-orbit orbit-back"></div>
    <div class="portal-ring"><span class="ring-streak"></span></div>
    <div class="portal-orbit orbit-front"></div>
    <div class="portal-core">
      <svg viewBox="0 0 220 240" aria-hidden="true">
        <defs>
          <linearGradient :id="id+'-top'" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#d4fff4"/><stop offset=".45" stop-color="#75ead7"/><stop offset="1" stop-color="#28a7ae"/></linearGradient>
          <linearGradient :id="id+'-left'" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#45bdb7"/><stop offset=".5" stop-color="#174556"/><stop offset="1" stop-color="#111c37"/></linearGradient>
          <linearGradient :id="id+'-right'" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#367d94"/><stop offset=".45" stop-color="#19394e"/><stop offset="1" stop-color="#13182b"/></linearGradient>
          <linearGradient :id="id+'-edge'" x1="0" y1="0" x2="0" y2="1"><stop stop-color="#d9fff6"/><stop offset=".7" stop-color="#6fffe06b"/><stop offset="1" stop-color="#7d8ffc10"/></linearGradient>
        </defs>
        <path d="m110 19 88 51-88 52-88-52Z" :fill="`url(#${id}-top)`"/>
        <path d="m22 70 88 52v102l-88-51Z" :fill="`url(#${id}-left)`"/>
        <path d="m110 122 88-52v103l-88 51Z" :fill="`url(#${id}-right)`"/>
        <path d="m110 19 88 51v103l-88 51-88-51V70l88-51Zm0 103v102M22 70l88 52 88-52" :stroke="`url(#${id}-edge)`" stroke-width="1.4" fill="none"/>
        <path d="m47 71 63-36 63 36-63 37Z" fill="#ffffff09" stroke="#eafff76b" stroke-width=".6"/>
        <path d="m73 72 37-21 37 21-37 21Z" fill="#083e4870" stroke="#c2fff2b0" stroke-width=".8"/>
        <path d="m100 68 17-9-4 12 12-6-25 22 4-13-12 6Z" fill="#e1fff5"/>
        <path d="m35 101 17 10v19l-17-10Zm28 16 17 10v19l-17-10Zm-28 27 17 10v19l-17-10Z" fill="#55d9cc12" stroke="#6cf3d529" stroke-width=".6"/>
        <path d="m132 139 43-25m-43 35 43-25m-43 35 43-25m-43 35 26-15" stroke="#70a2b75c" stroke-width="1"/>
        <path d="m24 73 86 50" stroke="#eefff7" stroke-width="1.7"/>
        <path d="m110 20 85 50" stroke="#e5fffbb0" stroke-width="1.4"/>
      </svg>
    </div>
    <div class="portal-platform"><i></i><i></i><i></i></div>
    <span v-for="i in 12" :key="i" class="portal-mote" :style="{'--n':i}"></span>
    <div class="satellite sat-one"><span></span></div><div class="satellite sat-two"><span></span></div><div class="satellite sat-three"><span></span></div>
    <span class="scene-axis axis-one"></span><span class="scene-axis axis-two"></span>
  </div>
  <div class="portal-corner corner-one"></div><div class="portal-corner corner-two"></div>
</div>
</template>
<style>
.portal-art{height:100%;width:100%;position:relative;overflow:hidden;isolation:isolate;background:radial-gradient(ellipse at 69% 43%,#2d797221,transparent 48%),radial-gradient(ellipse at 90% 90%,#4d35a127,transparent 60%),#0e141b;contain:paint}.portal-grid{position:absolute;inset:48% -20% -55%;background-image:linear-gradient(#87fff314 1px,transparent 1px),linear-gradient(90deg,#87fff314 1px,transparent 1px);background-size:42px 42px;transform:perspective(400px) rotateX(58deg) rotateZ(-13deg);mask-image:radial-gradient(ellipse,#0009,transparent 65%)}.portal-haze{position:absolute;inset:0;background:radial-gradient(ellipse at 65% 50%,#76ffe615,transparent 50%),linear-gradient(90deg,#10171f 2%,#10171fcc 28%,transparent 64%)}.portal-scene{position:absolute;width:390px;height:390px;right:0;top:50%;transform:translateY(-50%);transform-origin:center;pointer-events:none}.portal-ring{position:absolute;width:265px;height:265px;left:59px;top:43px;border:2px solid #a0fffcbf;border-radius:50%;transform:rotate(-28deg) rotateY(18deg);box-shadow:0 0 7px #aaffdf66,0 0 28px #5ce6dc20,inset 0 0 9px #7cf7f52b;z-index:1}.portal-ring:before{content:'';position:absolute;inset:-10px;border:1px solid #9bffeb30;border-radius:50%}.portal-ring:after{content:'';position:absolute;inset:10px;border:1px dashed #a4fff426;border-radius:50%}.ring-streak{position:absolute;inset:-3px;border:3px solid transparent;border-top-color:#e9fff8;border-left-color:#c1fff36b;border-radius:50%;animation:ring-journey 18s linear infinite}.portal-orbit{position:absolute;width:338px;height:106px;left:26px;top:151px;border:1px solid #b0a4ff57;border-radius:50%;transform:rotate(-26deg);z-index:0;box-shadow:0 0 12px #8d8cff05}.orbit-front{width:340px;height:90px;transform:rotate(24deg);border-color:#c7fff323;top:162px}.portal-core{position:absolute;width:186px;height:215px;left:102px;top:64px;z-index:2;animation:core-hover 7s ease-in-out infinite;filter:drop-shadow(0 16px 22px #0005)}.portal-core svg{height:100%;width:100%}.portal-platform{position:absolute;left:104px;top:302px;width:180px;height:28px;border:1px solid #6fffd557;border-radius:50%;background:radial-gradient(ellipse,#65ffda1c,transparent 65%);box-shadow:0 0 35px #76ffe60b;transform:rotate(-7deg)}.portal-platform i{position:absolute;inset:6px 35px;border:1px solid #70fff630;border-radius:50%}.portal-platform i:nth-child(2){inset:-10px -23px;border-color:#bdb3ff16}.portal-platform i:nth-child(3){inset:10px 65px;background:#b8ffe690;border:0}.portal-mote{position:absolute;left:calc(22px + var(--n)*27px);top:calc(35px + mod(var(--n)*47,290)*1px);width:2px;height:2px;background:#bafff4;border-radius:50%;opacity:.3;animation:mote-drift 6s ease-in-out infinite;animation-delay:calc(var(--n)*-.5s)}.satellite{position:absolute;width:27px;height:27px;background:linear-gradient(135deg,#adcfc9,#22545c 48%,#182a42 49%);border:1px solid #b1fff74d;border-radius:5px;transform:rotate(-25deg);box-shadow:0 8px 18px #0006;animation:satellite-drift 9s ease-in-out infinite;z-index:3}.satellite span{position:absolute;inset:6px;border:1px solid #e2fff94d;border-radius:2px}.sat-one{left:39px;top:115px;width:32px;height:32px}.sat-two{right:14px;top:252px;animation-delay:-3s;background:linear-gradient(135deg,#b3aae8,#5c4e82 48%,#232643 49%)}.sat-three{left:111px;bottom:21px;width:15px;height:15px;animation-delay:-6s;border-radius:3px}.sat-three span{inset:3px}.scene-axis{position:absolute;width:25px;height:1px;background:#9ef4ed45;left:312px;top:80px}.scene-axis:before{content:'';position:absolute;left:12px;top:-12px;width:1px;height:25px;background:inherit}.axis-two{left:25px;top:291px;transform:scale(.5)}.portal-corner{position:absolute;width:17px;height:17px;border-color:#b9fff42b;border-style:solid;pointer-events:none}.corner-one{right:20px;top:20px;border-width:1px 1px 0 0}.corner-two{left:20px;bottom:20px;border-width:0 0 1px 1px}@keyframes core-hover{0%,100%{transform:translateY(0) rotate(-3deg)}50%{transform:translateY(-13px) rotate(2deg)}}@keyframes ring-journey{to{transform:rotate(360deg)}}@keyframes mote-drift{0%,100%{transform:translateY(0);opacity:.15}50%{transform:translateY(-16px);opacity:.6}}@keyframes satellite-drift{0%,100%{transform:translateY(0) rotate(-25deg)}50%{transform:translateY(-9px) rotate(-15deg)}}
</style>
