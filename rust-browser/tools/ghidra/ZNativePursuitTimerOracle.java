// Numeric oracle for52ab0, using the real48470 target resolver.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePursuitTimerOracle extends GhidraScript{
 static final long U=0x5e30c8L,T=U+192,STACK=0x7fff0000L,RET=0x12345000L;
 void put(EmulatorHelper e,long a,int n,long v){e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){
  for(int ground=0;ground<2;ground++)for(int valid=0;valid<2;valid++)for(int actor:new int[]{0,1})for(int target:new int[]{0,1,2,4})for(int aim:new int[]{-128,-1,0,40,60,120,127})for(int[] d:new int[][]{{0,0},{200,100},{201,100},{500,0},{1000,0},{-32768,32767}}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[384]);put(e,U+2,1,aim);put(e,U+0x31,1,4);put(e,U+0x7c,1,actor);
    put(e,U+0x50,4,ground==1?0xd80:actor==1?0xffffef1fL:1);put(e,U+0x54,2,1);put(e,U+0x2c,2,0);put(e,U+0x2e,2,0);
    put(e,T+0x31,1,valid==1?5:1);put(e,T+0x7c,1,target);put(e,T+0xaa,1,0);put(e,T+0x2c,2,d[0]);put(e,T+0x2e,2,d[1]);
    e.writeRegister("ECX",U);e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x452ab0),null,monitor))throw new Exception(e.getLastError());
    int timer=e.readMemory(toAddr(U+0x68),1)[0];out.printf("%d,%d,%d,%d,%d,%d,%d,%d%n",ground,valid,actor,target,aim,d[0],d[1],timer);
   }finally{e.dispose();}
  }
 }}
}
