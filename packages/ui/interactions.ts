import {onMounted,onUnmounted} from 'vue';
export function useInteractions(){
  let controller:AbortController|null=null,observer:IntersectionObserver|null=null,changes:MutationObserver|null=null,frame=0;
  const timers=new Set<ReturnType<typeof setTimeout>>();
  onMounted(()=>{
    controller=new AbortController();const signal=controller.signal;
    const media=matchMedia('(prefers-reduced-motion: reduce)');
    const reduced=()=>media.matches||!!document.querySelector('[data-reduced-motion=true]');
    function visibleState(){document.documentElement.classList.toggle('motion-paused',document.hidden)}
    document.addEventListener('visibilitychange',visibleState,{signal});visibleState();
    observer=new IntersectionObserver(entries=>entries.forEach(entry=>entry.target.setAttribute('data-offscreen',String(!entry.isIntersecting))),{rootMargin:'60px'});
    const observed=new WeakSet<Element>();
    function observeArt(){document.querySelectorAll('.image-art,.skeleton-row').forEach(el=>{if(!observed.has(el)){observed.add(el);observer?.observe(el)}})}
    observeArt();changes=new MutationObserver(observeArt);changes.observe(document.getElementById('app')!,{childList:true,subtree:true});
    document.addEventListener('pointerdown',event=>{
      if(event.button!==0||reduced())return;
      const target=(event.target as Element).closest<HTMLElement>('button:not(:disabled),a.button');if(!target||target.closest('.window-buttons'))return;
      target.classList.remove('press-feedback');requestAnimationFrame(()=>target.classList.add('press-feedback'));
      const timer=setTimeout(()=>{target.classList.remove('press-feedback');timers.delete(timer)},450);timers.add(timer);
    },{signal});

  });
  onUnmounted(()=>{controller?.abort();observer?.disconnect();changes?.disconnect();cancelAnimationFrame(frame);timers.forEach(clearTimeout);document.documentElement.classList.remove('motion-paused')});
}
