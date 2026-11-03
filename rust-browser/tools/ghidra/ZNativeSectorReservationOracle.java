// Complete19240 including original17400 lookup, no call boundaries/stubs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeSectorReservationOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,G=0x5c47b8L,B=0x5c3f0cL;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);e.writeMemory(toAddr(G),new byte[240*24]);e.writeMemory(toAddr(B),new byte[64*30]);for(int i=0;i<64;i++)put(e,B+i*30,2,-1);for(int i=0;i<240;i++)put(e,R+i*72+19,1,255);put(e,B,2,0);put(e,B+2,2,0);put(e,B+4,2,(s&128)!=0?1:128);put(e,B+6,2,(s&128)!=0?1:128);put(e,0x4a69ea,1,1);
   int x=800+s%16,y=600+s/16%16;for(int i=0;i<3;i++){long r=R+i*72,u=U+i*3*192;put(e,G+i*24+21,1,(s>>i)&1);put(e,r+19,1,(s&(8<<i))!=0?255:i*3);put(e,r+58,1,(s/13+i)%3);put(e,r+44,2,x+new int[]{-32,-1,0,15,16}[(s+i)%5]);put(e,r+46,2,y+new int[]{-16,0,1,16}[(s/5+i)%4]);put(e,u+0x5e,1,new int[]{-1,0,1}[(s+i)%3]);put(e,u+0x5f,1,i);put(e,u+0x7c,1,(s/3+i)%4);put(e,u+0x31,1,s*7+i*5);put(e,u+0x35,1,(s/11+i)%4);}
   e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,2,y);e.writeRegister("EDX",x);if(!e.run(toAddr(0x419240),null,monitor))throw new Exception(e.getLastError());out.println("{\"scenario\":"+s+",\"reserved\":"+(e.readRegister("EAX").intValue()&255)+"}");
  }finally{e.dispose();}
 }}}
}
