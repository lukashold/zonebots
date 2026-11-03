//One actual point-fire handler callback; projectile/idle helpers are explicit
//boundaries. The caller supplies the raw phase/class/frame/timer/7b/78 fields.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePointFireOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L;int actions;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ESP",S);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<10000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;int action=pc==0x44ade0?1:pc==0x44ad40?2:pc==0x44abc0?3:pc==0x44afb0?4:pc==0x44cac0?5:pc==0x458620?6:pc==0x457120?7:0;if(action!=0){actions=actions*10+action;long sp=e.readRegister("ESP").longValue();e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int phase:new int[]{20,21,73,74,66})for(int cls:new int[]{0,15,16,3})for(int frame:new int[]{-1,0,1,2,3,4,5})for(int timer:new int[]{-128,-1,0,1,2,3})for(int cooldown:new int[]{0,1,100})for(int rate:new int[]{0,80,98}){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);put(e,U+0x35,1,phase);put(e,U+0x7c,1,cls);put(e,U+0x32,2,frame);put(e,U+0x3c,1,timer);put(e,U+0x7b,1,cooldown);put(e,U+0x78,1,rate);put(e,U+0x46,2,123);actions=0;call(e,phase==20?0x44af30:phase==21?0x44b020:phase==73?0x44b080:phase==74?0x44b0e0:0x4570d0);out.println(phase+","+cls+","+frame+","+timer+","+cooldown+","+rate+","+(short)read(e,U+0x32,2)+","+(byte)read(e,U+0x3c,1)+","+read(e,U+0x7b,1)+","+(short)read(e,U+0x46,2)+","+actions);out.flush();}finally{e.dispose();}
 }}}
}
