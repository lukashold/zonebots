// Executes448e80, including rawA0/5c/50 gates. Projectile creation,
//448750's result and the three cleanup helpers are explicit call boundaries.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeGroundFlameWindupOracle extends GhidraScript {
 static final long U=0x5e30c8L,G=0x60ec00L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{RET,0x456840,0x448750,0x45a9b0,0x452500,0x44cac0})e.setBreakpoint(toAddr(a));
  for(int frame:new int[]{-1,0,1,2,3,32767})for(int timer:new int[]{-128,-1,0,1,2,127})for(int cd:new int[]{0,1})for(int rate:new int[]{0,80,98,101,255})for(int cycle=0;cycle<4;cycle++)for(int order:new int[]{-5,0,5})for(int available=0;available<2;available++){
   e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(G),new byte[72]);put(e,0x632194,4,G);put(e,G+59,1,255);put(e,U+53,1,65);put(e,U+50,2,frame);put(e,U+60,1,timer);put(e,U+123,1,cd);put(e,U+120,1,rate);put(e,U+160,1,cycle);put(e,U+92,1,order);put(e,U+80,4,3456);put(e,U+94,1,-1);put(e,U+84,2,500);put(e,U+86,2,600);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ESP",S);e.writeRegister("ECX",U);int trace=0,shotTimer=-999;
   if(!e.run(toAddr(0x448e80),null,monitor))throw new Exception(e.getLastError());
   for(int guard=0;e.readRegister("EIP").longValue()!=RET;guard++){
    if(guard>16)throw new Exception("call budget");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();int event=pc==0x456840?1:pc==0x448750?2:pc==0x45a9b0?3:pc==0x452500?4:pc==0x44cac0?5:0;if(event==0)throw new Exception("unexpected "+Long.toHexString(pc));trace=trace*10+event;if(event==1)shotTimer=(byte)read(e,U+60,1);if(event==2)e.writeRegister("EAX",available);long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
   }
   out.println(frame+","+timer+","+cd+","+rate+","+cycle+","+order+","+available+","+(short)read(e,U+50,2)+","+(byte)read(e,U+60,1)+","+read(e,U+123,1)+","+read(e,U+160,1)+","+(byte)read(e,U+92,1)+","+trace+","+shotTimer);
  }
 }finally{e.dispose();}}}
}
