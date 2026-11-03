// Runs native road endpoint selection0043b090 and road-width alignment43ae30.
// Uses only synthetic road/permission planes.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRoadEndpointOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]points={{512,512},{513,525},{16,16},{1000,900},{0,0},{2047,2047},{960,640},{320,640}};
  for(int mode=0;mode<4;mode++)for(int index=0;index<points.length;index++)for(int flagged=0;flagged<2;flagged++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++){int code=(x+y)%9==0?1:0;boolean r=mode==0?(y>=40&&y<=41&&x>=10&&x<=100||x>=60&&x<=63&&y>=10&&y<=100):mode==1?x%16==0||y%16==0:mode==3?x%15==0||y%15==0:false;if(r)code=mode==3?new int[]{7,23,51}[(x+y)%3]:7;map[y*128+x]=(byte)code;}e.writeMemory(toAddr(0x58aaaa),map);e.writeMemory(toAddr(U),new byte[0x2000]);put(e,T+1,1,flagged==1?255:0);put(e,U+0x18,4,T);put(e,0x622d50,2,2048);put(e,0x622d52,2,2048);put(e,S,4,RET);put(e,S+4,4,points[index][1]);put(e,S+8,4,U);put(e,S+12,4,U+2);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EDX",points[index][0]);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x43b090),null,monitor))throw new Exception(e.getLastError());
    out.println("{\"mode\":"+mode+",\"index\":"+index+",\"flagged\":"+flagged+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"x\":"+(short)read(e,U,2)+",\"y\":"+(short)read(e,U+2,2)+",\"onRoad\":"+read(e,0x4d0273,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
