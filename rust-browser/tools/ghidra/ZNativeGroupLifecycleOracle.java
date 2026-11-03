// Execute the successful4c0b0→4bba0→4b2f0→4b7b0 allocator chain. None of these
// routines is intercepted. The generated raw roster/strategy fixtures are hashed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeGroupLifecycleOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,G=0x5c47b8L,B=0x5c3f0cL,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(int h,int n){return(h^n)*16777619;}
 void free(EmulatorHelper e,int slot,int cooldown)throws Exception{put(e,R+slot*72+18,1,0);put(e,R+slot*72+19,1,255);put(e,0x586880+slot,1,cooldown);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(0x612f80),new byte[200*44]);
    for(int i=0;i<64;i++){e.writeMemory(toAddr(B+i*30),new byte[30]);put(e,B+i*30,2,65535);}put(e,B,2,0);put(e,B+4,2,8);put(e,B+6,2,8);
    for(int i=0;i<240;i++){
     byte[]r=new byte[72],g=new byte[24];for(int j=0;j<72;j++)r[j]=(byte)(scenario*7+i*3+j);for(int j=0;j<24;j++)g[j]=(byte)(scenario*11+i*5+j);e.writeMemory(toAddr(R+i*72),r);e.writeMemory(toAddr(G+i*24),g);
     put(e,R+i*72+18,1,1);for(int m=0;m<21;m++)put(e,R+i*72+19+m,1,m==0?200:255);put(e,0x586880+i,1,0);
    }
    int mode=scenario%8,count=2+scenario%20,selected=count-1;
    if(mode==0||mode==1||mode==7){free(e,0,0);free(e,121,mode==1?7:0);free(e,122,0);}
    else if(mode==2){free(e,0,7);free(e,125,7);}else if(mode==3)free(e,125,7);
    if(mode==5)count=1;
    if(mode==7){count=2;selected=20;}
    put(e,R+120*72+18,1,count);for(int m=0;m<21;m++)put(e,R+120*72+19+m,1,m==0?0:mode==6?255:mode==7?(m==20?20:255):(m<count?m:255));
    for(int i=0;i<21;i++){long u=U+i*192;put(e,u+0x31,1,((scenario&8)!=0?0:1)|4|((scenario&32)!=0?32:0)|((scenario&4)!=0?8:0));put(e,u+0x5f,1,120);put(e,u+0x5e,1,i==0?255:i);put(e,u+0x69,1,255);put(e,u+0xb7,1,159);put(e,u+0xa8,2,0x1234);put(e,u+0x5d,1,7);put(e,u+0x2c,2,32+i*2);put(e,u+0x2e,2,64);if((scenario&1)!=0){put(e,u+0x14,4,0x5eee28+i*8);put(e,0x5eee28+i*8+4,4,0);}if((scenario&2)!=0){put(e,u+12,4,0x5f8428+i*8);put(e,0x5f8428+i*8+4,4,0);}}
    put(e,0x612f8d,1,255);put(e,0x584963,1,0);put(e,0x622dc4,1,(scenario&16)!=0?1:0);put(e,0x63238c,4,0x11223344);
    e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",R+120*72);if(!e.run(toAddr(0x44c0b0),null,monitor))throw new Exception(e.getLastError());
    long result=e.readRegister("EAX").longValue();int member=result==0?-1:(int)((result-U)/192),rh=0x811c9dc5,sh=0x811c9dc5,ch=0x811c9dc5,mh=0x811c9dc5;
    for(int i=0;i<240;i++){for(int j=0;j<72;j++)rh=hash(rh,(int)read(e,R+i*72+j,1));for(int j=0;j<24;j++)sh=hash(sh,(int)read(e,G+i*24+j,1));ch=hash(ch,(int)read(e,0x586880+i,1));}
    for(int i=0;i<21;i++){long u=U+i*192;int[]values={i,(int)read(e,u+0x5f,1),(byte)read(e,u+0x5e,1),(int)read(e,u+0x31,1),(int)read(e,u+0x69,1),(int)read(e,u+0xb7,1),read(e,u+0x14,4)!=0?1:0,read(e,u+12,4)!=0?1:0,(int)read(e,u+0xa8,2),(int)read(e,u+0x5d,1),(short)read(e,u+0x2c,2),(short)read(e,u+0x2e,2)};for(int n:values)mh=hash(mh,n);}
    out.println("{\"scenario\":"+scenario+",\"member\":"+member+",\"group\":"+(member<0?-1:read(e,U+member*192+0x5f,1))+",\"role\":"+(member<0?0:(byte)read(e,U+member*192+0x5e,1))+",\"roster\":"+Integer.toUnsignedString(rh)+",\"strategy\":"+Integer.toUnsignedString(sh)+",\"cooldown\":"+Integer.toUnsignedString(ch)+",\"members\":"+Integer.toUnsignedString(mh)+",\"reservation\":"+read(e,0x612f8d,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
