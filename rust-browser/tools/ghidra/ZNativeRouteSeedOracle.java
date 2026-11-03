// Native local contour seed functions with explicit contact slots and masks.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteSeedOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,P=U+0x2000,M=U+0x4000,C=0x4d15f0L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[]offsets={-129,-128,-127,-2,-1,0,1,2,127,128,129};long[]foot={0x43c4c0,0x43c5a0,0x43c770,0x43c680},vehicleEntries={0x43da20,0x43d7f0,0x43d8a0,0x43d960};
  for(int vehicle=0;vehicle<2;vehicle++)for(int direction=0;direction<4;direction++)for(int offset:offsets)for(int steps:new int[]{0,1,256,257})for(int blocked=0;blocked<2;blocked++)for(int valid=0;valid<2;valid++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x9000]);e.writeMemory(toAddr(C),new byte[400]);byte[]map=new byte[16384],mask=new byte[16384];for(int i=0;i<16384;i++){map[i]=(byte)(blocked==1&&Math.abs(i-4112)<=1?1:0);mask[i]=65;}e.writeMemory(toAddr(0x58aaaa),map);e.writeMemory(toAddr(M),mask);put(e,T+1,1,255);put(e,U+0x18,4,T);put(e,U+0x58,2,valid==1?4112:-1);put(e,U+0x5a,2,valid==1?4112:-1);put(e,0x63288c,4,U);put(e,0x633948,4,M);put(e,0x4d15e4,2,4112+offset);put(e,P+4,4,steps);put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EDX",P);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(vehicle==1?vehicleEntries[direction]:foot[direction]),null,monitor))throw new Exception(e.getLastError());int failure=e.readRegister("EAX").intValue()&255;long header=C+direction*100;int count=(byte)read(e,header+1,1);StringBuilder points=new StringBuilder();if(failure==0)for(int i=0;i<count;i++){if(i>0)points.append(',');points.append((short)read(e,header+2+i*2,2));}
    out.println("{\"vehicle\":"+vehicle+",\"direction\":"+direction+",\"offset\":"+offset+",\"steps\":"+steps+",\"blocked\":"+blocked+",\"valid\":"+valid+",\"failure\":"+failure+",\"cell\":"+(failure==0?(int)read(e,P+0x14,4):0)+",\"wall\":"+(failure==0?(int)read(e,P+0x18,4):0)+",\"step\":"+(failure==0?(int)read(e,P+0x10,4):0)+",\"nextSteps\":"+(failure==0?read(e,P+4,4):0)+",\"score\":"+(failure==0?read(e,header,1):0)+",\"count\":"+(failure==0?count:0)+",\"points\":["+points+"]}");
   }finally{e.dispose();}
  }
 }}
}
