// Executes complete native global road candidate0043bbe0 local replacement0043df20 with synthetic MAP
// records/terrain and native0043ba60 comparison. No routing functions stubbed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZRound44LongScratchOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,N=0x4d1480L,P=U+0x2000,G=0x5b6da4L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int rng,oldCost,newCost;String initialized,built;
 String hex(EmulatorHelper e,long a,int n){StringBuilder b=new StringBuilder();for(byte v:e.readMemory(toAddr(a),n))b.append(String.format("%02x",v&255));return b.toString();}
 void execute(EmulatorHelper e,long entry,long input)throws Exception{rng=0;oldCost=newCost=-1;initialized=built="";for(long a:new long[]{RET,0x43dfcd,0x43dff1,0x43e02c,0x43e0a3,0x460910})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",input);if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());int n=0;while(e.readRegister("EIP").longValue()!=RET){if(++n>1000)throw new Exception("eventbudget");long pc=e.readRegister("EIP").longValue();if(pc==0x460910)rng++;if(pc==0x43dfcd)oldCost=e.readRegister("EAX").intValue();if(pc==0x43dff1)initialized=hex(e,N,640);if(pc==0x43e02c)built=hex(e,N,640);if(pc==0x43e0a3)newCost=e.readRegister("EAX").intValue();if(!e.step(monitor)||!e.run(monitor))throw new Exception(e.getLastError());}}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]starts={{160,656},{160,500},{500,160},{1120,656},{960,880}},goals={{1120,656},{1120,500},{1000,880},{176,672},{170,656},{1200,1300}};
  for(int hops:new int[]{8,30,32,33,34,35,38,39,40,41,42,43,44,45,46,47,50,60,79,80})for(int warm=0;warm<2;warm++)for(int mode=0;mode<1;mode++)for(int start=0;start<1;start++)for(int goal=0;goal<1;goal++)for(int vehicle=0;vehicle<2;vehicle++)for(int compare=0;compare<2;compare++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{starts[0]=new int[]{136,656};goals[0]=new int[]{(8+hops)*16+8,656};
    e.writeMemory(toAddr(U),new byte[0x3000]);e.writeMemory(toAddr(G),new byte[96*136]);byte[]raw=new byte[800];for(int k=0;k<raw.length;k++)raw[k]=(byte)(warm==0?0:255);e.writeMemory(toAddr(N),raw);e.writeMemory(toAddr(0x4d1340),new byte[400]);put(e,0x622d50,2,2048);put(e,0x622d52,2,2048);put(e,0x622c7a,2,mode==1?5:0);put(e,0x622dc4,1,0);put(e,0x4d0e33,1,0);put(e,0x4d0272,1,compare==0?1:0);put(e,U+0x18,4,T);put(e,U+0x7c,1,vehicle);put(e,U+0x2c,2,starts[start][0]);put(e,U+0x2e,2,starts[start][1]);put(e,U+0x5e,1,0);
    byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++){int code=mode==1&&x>=40&&x<=42?40:0;if(y>=40&&y<=41&&x>=8&&x<=100)code=7;map[y*128+x]=(byte)code;}e.writeMemory(toAddr(0x58aaaa),map);put(e,T+40,1,255);byte[]cost=new byte[128];for(int i=0;i<128;i++)cost[i]=5;cost[7]=2;cost[40]=72;e.writeMemory(toAddr(0x499b98),cost);e.writeMemory(toAddr(0x499ec8),cost);
    for(int i=0;i<96;i++){long r=G+i*136;for(int j=0;j<96;j++)put(e,r+40+j,1,255);if(i<hops+1){put(e,r,2,(8+i)*16);put(e,r+2,2,32*16);put(e,r+4,2,(9+i)*16-1);put(e,r+6,2,50*16);put(e,r+8,2,(8+i)+40*128);for(int j=0;j<hops+1;j++)put(e,r+40+j,1,i==j?j:i+(j>i?1:-1));}}
    put(e,U+0x9c,1,compare==1?2:0);put(e,0x63288c,4,U);put(e,0x4d15e4,4,5256);put(e,0x4d15e8,4,1);put(e,0x633a14,4,vehicle==1?0x499ec8:0x499b98);
    int sx=starts[start][0]>>4,gx=goals[goal][0]>>4,gy=goals[goal][1]>>4;int[]cells={sx+20*128,gx+20*128,gx+gy*128};put(e,P+1,1,-cells.length);for(int i=0;i<cells.length;i++)put(e,P+2+i*2,2,cells[i]);
    String before=hex(e,N,640);execute(e,0x43df20,P);int use=e.readRegister("EAX").intValue()&255;StringBuilder points=new StringBuilder();long p=N;int count=0;if(use!=0)while(p!=0&&p!=-1&&count<99){if(count++>0)points.append(',');points.append('[').append((short)read(e,p,2)).append(',').append((short)read(e,p+2,2)).append(']');p=read(e,p+4,4);}
    out.println("{\"hops\":"+hops+",\"warm\":"+warm+",\"rng\":"+rng+",\"oldCost\":"+oldCost+",\"newCost\":"+newCost+",\"unitGlobal\":"+read(e,0x4d15e0,4)+",\"originGlobal\":"+read(e,0x4d15e4,4)+",\"goalGlobal\":"+read(e,0x4d15ec,4)+",\"scratchMode\":"+read(e,0x4d0e33,1)+",\"compareMode\":"+read(e,0x4d0272,1)+",\"recorded\":"+read(e,0x586867,1)+",\"recordCursor\":"+read(e,0x633a24,4)+",\"before\":\""+before+"\",\"initialized\":\""+initialized+"\",\"built\":\""+built+"\",\"after\":\""+hex(e,N,640)+"\",\"mode\":"+mode+",\"start\":"+start+",\"goal\":"+goal+",\"vehicle\":"+vehicle+",\"suppressed\":"+compare+",\"use\":"+use+",\"count\":"+count+",\"points\":["+points+"]}");out.flush();
   }finally{e.dispose();}
  }
 }}
}
