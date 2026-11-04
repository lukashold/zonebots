// Exact19/49 callback counters. Projectile creation,58500 and idle reset
//are explicit boundaries;58500 returnsFFFF (no live target).
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeGroundAuxiliaryWindupOracle extends ZNativeGroundFlameWindupOracle {
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{RET,0x457120,0x457300,0x458500,0x44cac0})e.setBreakpoint(toAddr(a));
  for(int phase:new int[]{19,49})for(int frame:new int[]{-1,0,1,2,3,32767})for(int timer:new int[]{-128,-1,0,1,2,127})for(int cd:new int[]{0,1})for(int rate:new int[]{0,80,98,101,255})for(int order:new int[]{-5,0,5}){
   e.writeMemory(toAddr(U),new byte[192]);put(e,U+53,1,phase);put(e,U+50,2,frame);put(e,U+60,1,timer);put(e,U+123,1,cd);put(e,U+120,1,rate);put(e,U+92,1,order);put(e,U+80,4,3456);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ESP",S);e.writeRegister("ECX",U);int trace=0,shotTimer=-999,shotCooldown=-999;
   if(!e.run(toAddr(phase==19?0x457400:0x457270),null,monitor))throw new Exception(e.getLastError());
   for(int guard=0;e.readRegister("EIP").longValue()!=RET;guard++){
    if(guard>8)throw new Exception("call budget");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();int event=pc==0x457120||pc==0x457300?1:pc==0x44cac0?5:pc==0x458500?6:0;if(event==0)throw new Exception("unexpected "+Long.toHexString(pc));trace=trace*10+event;if(event==1){shotTimer=(byte)read(e,U+60,1);shotCooldown=(int)read(e,U+123,1);}if(event==6)e.writeRegister("EAX",65535);long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
   }
   out.println(phase+","+frame+","+timer+","+cd+","+rate+","+order+","+(short)read(e,U+50,2)+","+(byte)read(e,U+60,1)+","+read(e,U+123,1)+","+(byte)read(e,U+92,1)+","+trace+","+shotTimer+","+shotCooldown);
  }
 }finally{e.dispose();}}}
}
