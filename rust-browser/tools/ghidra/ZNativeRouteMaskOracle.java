// Native0043cb20 ray masks and004517c0 integer direction setup.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteMaskOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,N=U+0x2000,M=U+0x4000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]starts={{464,512},{513,525},{512,304}},goals={{648,520},{648,648},{168,520},{512,824}};
  for(int vehicle=0;vehicle<2;vehicle++)for(int alternate=0;alternate<2;alternate++)for(int grace:new int[]{0,1,2,127})for(int start=0;start<starts.length;start++)for(int goal=0;goal<goals.length;goal++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x9000]);byte[]map=new byte[16384],mask=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++){map[y*128+x]=(byte)(x>=30&&x<=34&&y>=20&&y<=40?1:0);mask[y*128+x]=65;}e.writeMemory(toAddr(0x58aaaa),map);e.writeMemory(toAddr(M),mask);put(e,U+0x18,4,T);put(e,T+1,1,255);put(e,U+0x31,1,4);put(e,U+0x7c,1,vehicle);put(e,U+0xb4,1,alternate);put(e,U+0x2c,2,starts[start][0]);put(e,U+0x2e,2,starts[start][1]);put(e,U+0x14,4,N);put(e,N,2,goals[goal][0]);put(e,N+2,2,goals[goal][1]);put(e,0x633948,4,M);put(e,0x63288c,4,U);put(e,0x586862,1,grace);e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x43cb20),null,monitor))throw new Exception(e.getLastError());long hash=2166136261L;byte[]visited=e.readMemory(toAddr(M),16384);for(byte b:visited)hash=((hash^(b&255L))*16777619L)&0xffffffffL;
    out.println("{\"vehicle\":"+vehicle+",\"alternate\":"+alternate+",\"grace\":"+grace+",\"start\":"+start+",\"goal\":"+goal+",\"status\":"+(e.readRegister("EAX").intValue()&255)+",\"maskHash\":"+hash+"}");
   }finally{e.dispose();}
  }
 }}
}
