// Execute real4b990 promotion and52500 path cleanup; RNG alone is deterministic.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePromotionOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,P=0x5eee28L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(int h,int n){return(h^n)*16777619;}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);int rng=123+scenario,calls=0;
   try{
    e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[72*240]);long g=R+120*72;byte[]raw=new byte[72];for(int j=0;j<72;j++)raw[j]=(byte)(scenario*7+j);e.writeMemory(toAddr(g),raw);for(int m=0;m<21;m++)put(e,g+19+m,1,255);
    int mode=scenario%4;put(e,g+18,1,mode==3?0:mode==2?2:3);if(mode!=3){put(e,g+19+(mode==0?1:mode==1?2:5),1,mode==2?1:3);if(mode!=2)put(e,g+19+5,1,1);put(e,g+19+20,1,2);}
    long[]paths={0,0xffffffffL,P+800,P+0x12c0*8,P+0x12c1*8,P-8,P+960,P+1};long path=paths[scenario%8];
    for(int i=0;i<5;i++){
     long u=U+i*192;put(e,u+0x31,1,i==0?0:5|32);put(e,u+0x5e,1,i==0?0:i==1?5:i==2?20:mode==0?1:2);put(e,u+0x5f,1,i==0?255:120);put(e,u+0x2c,2,32+i*2);put(e,u+0x2e,2,64);
     long own=i==0?path:P+1600+i*8;if(i==4&&(scenario%8)==6)own=path;put(e,u+0x14,4,own);if(own>=P&&own<=P+0x12c0*8)put(e,own+4,4,0);put(e,u+12,4,0x5f8428+i*8);put(e,0x5f8428+i*8+4,4,0);
     put(e,u+0x50,4,i==0?0x12348000L+scenario*333:100000+i);put(e,u+0x54,2,i==0?-300+scenario*5:100+i);put(e,u+0x56,2,i==0?1000+scenario*7:200+i);put(e,u+0x5c,1,i==0?17+scenario:33);put(e,u+0x61,1,i==0?99-scenario:44);put(e,u+0x35,1,i!=0&&(scenario&8)!=0?7:1);put(e,u+0x32,2,-7);put(e,u+0x39,1,10+i);put(e,u+0x3c,1,21);put(e,u+0x3d,1,22);put(e,u+0x40,1,23);put(e,u+0x41,1,24);put(e,u+0x7d,1,25);
    }
    put(e,0x622d1a,2,0);put(e,0x6338e8,2,0);put(e,0x632158,4,0);for(long a:new long[]{RET,0x460910})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,0);e.writeRegister("ECX",U);e.writeRegister("EDX",g);if(!e.run(toAddr(0x44b990),null,monitor))throw new Exception(e.getLastError());
    while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x460910)throw new Exception("Unexpected promotion stop");rng=rng*1664525+1013904223;calls++;e.writeRegister("EAX",rng);resume(e);}
    int rh=0x811c9dc5,mh=0x811c9dc5;for(int j=0;j<72;j++)rh=hash(rh,(int)read(e,g+j,1));
    for(int i=0;i<5;i++){long u=U+i*192;int[]v={i,(int)read(e,u+0x5f,1),(byte)read(e,u+0x5e,1),(int)read(e,u+0x31,1),read(e,u+0x14,4)!=0?1:0,read(e,u+12,4)!=0?1:0,(int)read(e,u+0x14,4),(int)read(e,u+0x50,4),(short)read(e,u+0x54,2),(short)read(e,u+0x56,2),(int)read(e,u+0x5c,1),(int)read(e,u+0x61,1),(int)read(e,u+0x35,1),(short)read(e,u+0x32,2),(int)read(e,u+0x39,1),(int)read(e,u+0x3c,1),(int)read(e,u+0x3d,1),(int)read(e,u+0x40,1),(int)read(e,u+0x41,1),(int)read(e,u+0x7d,1)};for(int n:v)mh=hash(mh,n);}
    out.println("{\"scenario\":"+scenario+",\"promoted\":"+((e.readRegister("EAX").longValue()&255)==0?read(e,g+19,1):-1)+",\"roster\":"+Integer.toUnsignedString(rh)+",\"members\":"+Integer.toUnsignedString(mh)+",\"rng\":"+calls+"}");
   }finally{e.dispose();}
  }
 }}
}
