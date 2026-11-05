// Own harness: complete43bbe0 with fragmented finite pools; routing has no stubs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class RootV149PrimaryRoadRasterOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,G=0x5b6da4L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void set(byte[]b,int i,int n,long v){for(int k=0;k<n;k++)b[i+k]=(byte)(v>>(k*8));}
 long mix(long h,long v){return((h^(v&0xffffffffL))*16777619L)&0xffffffffL;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]starts={{160,656},{160,500},{1120,656},{500,160}},goals={{1120,656},{1120,500},{176,672},{1200,1300}};
  for(int c=0;c<288;c++){
   int mode=c%3,layout=c/3%12,compare=c/36%2,pair=c/72;
   boolean scratch=layout>=8;if(scratch)continue;int count=scratch?80:14400,headIndex=new int[]{0,0,0,4796,4798,4799,4800,0,0,76,78,79}[layout];
   long base=scratch?0x4d1480L:0x5eee28L,head=base+headIndex*8;
   boolean longRoad=mode==2&&!scratch;int[]start=longRoad?new int[]{160,656}:starts[pair],goal=longRoad?new int[]{1650,656}:goals[pair];
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    long[] words={0x6330a8,0x6330aa,0x6330ac,0x6330b0,0x6330b2,0x6330b4,0x62d5d0,0x62d5d4,0x62d5dc,0x632724,0x632574,0x632604,0x63276c,0x6326dc,0x63264c,0x6325bc,0x4d2be4};
    for(String reg:new String[]{"EAX","EBX","ECX","EDX","ESI","EDI","EBP","ESP","DF"})e.writeRegister(reg,0);e.writeMemory(toAddr(S-4096),new byte[4096]);
    for(int j=0;j<words.length;j++)put(e,words[j],2,0x3b00+c%13*64+j);put(e,0x632694,1,9);
    e.writeMemory(toAddr(U),new byte[0x2000]);e.writeMemory(toAddr(G),new byte[96*136]);e.writeMemory(toAddr(0x4d1340),new byte[320]);
    byte[]nodes=new byte[count*8];for(int i=0;i<count;i++){set(nodes,i*8,2,100+i%200);set(nodes,i*8+2,2,200+i%300);long next=(layout==1&&i%3!=0||layout==2&&i>2||layout==7&&i!=0&&i!=4800)?0:0xffffffffL;set(nodes,i*8+4,4,next);}set(nodes,headIndex*8,2,goal[0]);set(nodes,headIndex*8+2,2,goal[1]);set(nodes,headIndex*8+4,4,0);e.writeMemory(toAddr(base),nodes);
    put(e,0x622d50,2,2048);put(e,0x622d52,2,2048);put(e,0x622c7a,2,mode==1?5:0);put(e,0x622dc4,1,0);put(e,0x4d0e33,1,scratch?1:0);put(e,0x4d0272,1,compare==0?1:0);put(e,U+0x18,4,T);put(e,U+0x7c,1,c%2);put(e,U+0x2c,2,start[0]);put(e,U+0x2e,2,start[1]);put(e,U+0x5e,1,0);put(e,U+0x9c,1,3);
    byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++){int code=mode==1&&x>=40&&x<=42?40:0;if(y>=40&&y<=41&&x>=8&&x<=110)code=7;map[y*128+x]=(byte)code;}e.writeMemory(toAddr(0x58aaaa),map);put(e,T+40,1,255);byte[]cost=new byte[128];java.util.Arrays.fill(cost,(byte)5);cost[7]=2;cost[40]=72;e.writeMemory(toAddr(0x499b98),cost);e.writeMemory(toAddr(0x499ec8),cost);
    for(int i=0;i<96;i++){long r=G+i*136;for(int j=0;j<96;j++)put(e,r+40+j,1,255);int n=longRoad?96:7;if(i<n){int left=longRoad?8+i:8+i*12,right=longRoad?9+i:20+i*12;put(e,r,2,left*16);put(e,r+2,2,32*16);put(e,r+4,2,right*16-1);put(e,r+6,2,50*16);put(e,r+8,2,(longRoad?8+i:14+i*12)+40*128);for(int j=0;j<n;j++)put(e,r+40+j,1,i==j?j:i+(j>i?1:-1));}}
    e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",head);e.writeRegister("EDX",head);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x43bbe0),null,monitor))throw new Exception(e.getLastError());
    StringBuilder raster=new StringBuilder("[");for(long key:words)raster.append((short)read(e,key,2)).append(',');raster.append(read(e,0x632694,1)).append(']');long hash=2166136261L;for(byte v:e.readMemory(toAddr(base),count*8))hash=mix(hash,v&255);long links=2166136261L,p=head;int length=0;while(p!=0&&p!=0xffffffffL&&length<200){links=mix(links,p);length++;p=read(e,p+4,4);}out.println("{\"case\":"+c+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"recorded\":"+read(e,0x586867,1)+",\"flags\":"+read(e,U+0x9c,1)+",\"pool\":"+hash+",\"links\":"+links+",\"length\":"+length+",\"raster\":"+raster+"}");
   }finally{e.dispose();}
  }
 }}
}
