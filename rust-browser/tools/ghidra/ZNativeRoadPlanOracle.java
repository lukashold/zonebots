// Executes complete native global road candidate0043bbe0 with synthetic MAP
// records/terrain and native0043ba60 comparison. No routing functions stubbed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRoadPlanOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,N=0x5eee28L,G=0x5b6da4L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]starts={{160,656},{160,500},{500,160},{1120,656},{960,880}},goals={{1120,656},{1120,500},{1000,880},{176,672},{170,656},{1200,1300}};
  for(int mode=0;mode<2;mode++)for(int start=0;start<starts.length;start++)for(int goal=0;goal<goals.length;goal++)for(int vehicle=0;vehicle<2;vehicle++)for(int compare=0;compare<2;compare++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);e.writeMemory(toAddr(G),new byte[96*136]);e.writeMemory(toAddr(N),new byte[100*8]);e.writeMemory(toAddr(0x4d1340),new byte[400]);put(e,0x622d50,2,2048);put(e,0x622d52,2,2048);put(e,0x622c7a,2,mode==1?5:0);put(e,0x622dc4,1,0);put(e,0x4d0e33,1,0);put(e,0x4d0272,1,compare==0?1:0);put(e,U+0x18,4,T);put(e,U+0x7c,1,vehicle);put(e,U+0x2c,2,starts[start][0]);put(e,U+0x2e,2,starts[start][1]);put(e,U+0x5e,1,0);
    byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++){int code=mode==1&&x>=40&&x<=42?40:0;if(y>=40&&y<=41&&x>=8&&x<=100)code=7;map[y*128+x]=(byte)code;}e.writeMemory(toAddr(0x58aaaa),map);put(e,T+40,1,255);byte[]cost=new byte[128];for(int i=0;i<128;i++)cost[i]=5;cost[7]=2;cost[40]=72;e.writeMemory(toAddr(0x499b98),cost);e.writeMemory(toAddr(0x499ec8),cost);
    for(int i=0;i<96;i++){long r=G+i*136;for(int j=0;j<96;j++)put(e,r+40+j,1,255);if(i<7){put(e,r,2,(8+i*12)*16);put(e,r+2,2,32*16);put(e,r+4,2,(20+i*12)*16-1);put(e,r+6,2,50*16);put(e,r+8,2,(14+i*12)+40*128);for(int j=0;j<7;j++)put(e,r+40+j,1,i==j?j:i+(j>i?1:-1));}}
    put(e,N,2,goals[goal][0]);put(e,N+2,2,goals[goal][1]);for(int i=1;i<100;i++)put(e,N+i*8+4,4,-1);e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",N);e.writeRegister("EDX",N);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x43bbe0),null,monitor))throw new Exception(e.getLastError());StringBuilder points=new StringBuilder();long p=N;int count=0;while(p!=0&&p!=-1&&count<99){if(count++>0)points.append(',');points.append('[').append((short)read(e,p,2)).append(',').append((short)read(e,p+2,2)).append(']');p=read(e,p+4,4);}
    out.println("{\"mode\":"+mode+",\"start\":"+start+",\"goal\":"+goal+",\"vehicle\":"+vehicle+",\"compare\":"+compare+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"count\":"+count+",\"points\":["+points+"]}");
   }finally{e.dispose();}
  }
 }}
}
