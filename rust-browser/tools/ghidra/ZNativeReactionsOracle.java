// Run the native projectile-cover predicates and pose counters. External
// acquisition/reset callbacks are intercepted only when testing their caller.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeReactionsOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,P=0x1100000L,G=0x60ec00L;
 int blocker,probe,calls,randomIndex;boolean hit,finished;int[]random=new int[2];
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void run(EmulatorHelper e,long a,long ecx,long edx)throws Exception{
  hit=false;finished=false;calls=0;randomIndex=0;e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);
  e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue();
   if(pc==0x44cd60){put(e,0x62d3a8,1,0);put(e,0x62d3a9,1,255);}
   else if(pc==0x459250){hit=true;e.writeRegister("EAX",0);}
   else if(pc==0x458590){calls++;e.writeRegister("EAX",blocker);}
   else if(pc==0x458500){calls++;e.writeRegister("EAX",probe);}
   else if(pc==0x44cac0){finished=true;}
   else if(pc==0x460910){e.writeRegister("EAX",random[randomIndex++]);}
   else throw new Exception("Unexpected stop "+Long.toHexString(pc));
   long stack=e.readRegister("ESP").longValue(),ret=read(e,stack,4);e.writeRegister("ESP",stack+4);
   if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 void output(PrintWriter out,EmulatorHelper e,String prefix)throws Exception{
  out.println(prefix+",\"state\":"+read(e,U+0x35,1)+",\"frame\":"+(short)read(e,U+0x32,2)+",\"timer82\":"+(short)read(e,U+0x82,2)+",\"animation89\":"+read(e,U+0x89,1)+",\"counter7e\":"+(short)read(e,U+0x7e,2)+",\"remaining80\":"+read(e,U+0x80,1)+"}");
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  EmulatorHelper e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(0x44cd60));e.setBreakpoint(toAddr(0x459250));int scenario=0;
   for(int cls:new int[]{0,1,2,3,15,16,17,18,255})for(int state:new int[]{0,1,2,43,255})for(int[]delta:new int[][]{{0,0},{7,0},{8,0},{5,6},{-7,0},{0,-8},{32767,0},{-32768,0},{32767,32767},{-32768,-32768}}){
    e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(P),new byte[0x40]);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x7c,1,cls);put(e,U+0x35,1,state);put(e,P,2,500+delta[0]);put(e,P+2,2,500+delta[1]);
    run(e,0x462930,P,0);out.println("{\"kind\":\"near\",\"scenario\":"+scenario+++",\"hit\":"+(hit?1:0)+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(0x458590));int scenario=0;
   for(int code=0;code<128;code++)for(int facing=0;facing<16;facing++)for(blocker=0;blocker<2;blocker++){
    e.writeMemory(toAddr(U),new byte[0xc0]);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x39,1,facing);put(e,0x58aaaa+3999,1,code);
    put(e,U+0x35,1,1);put(e,U+0x32,2,-123);put(e,U+0x82,2,-456);put(e,U+0x89,1,77);put(e,U+0x7e,2,-1234);put(e,U+0x80,1,63);
    run(e,0x459250,U,0);output(out,e,"{\"kind\":\"begin\",\"scenario\":"+scenario+++",\"result\":"+(e.readRegister("EAX").intValue()&255)+",\"calls\":"+calls);
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(0x458500));int[]probes={0xff00,0,1,15,127,255,0xff01,0x0107},members={-1,0,1,127};
   for(int scenario=0;scenario<4096;scenario++){
    e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(G),new byte[0x48]);put(e,U+0x5e,1,members[scenario&3]);put(e,U+0x5f,1,(scenario&4)!=0?255:0);put(e,U+4,1,scenario*43);
    put(e,G+9,1,(scenario>>3)&1);put(e,G+0x3d,1,(scenario>>4)&1);put(e,G+0x3a,1,(scenario>>5)&1);put(e,0x4d0e3e,1,(scenario>>6)&1);put(e,G+0x10,1,(scenario>>7)&1);put(e,G+0x12,1,(scenario>>8)&1);probe=probes[(scenario>>9)&7];
    run(e,0x458590,U,0);out.println("{\"kind\":\"group\",\"scenario\":"+scenario+",\"result\":"+(e.readRegister("EAX").intValue()&255)+",\"calls\":"+calls+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(0x458590));e.setBreakpoint(toAddr(0x44cac0));int scenario=0;
   for(int facing=0;facing<16;facing++)for(int phase=0;phase<2;phase++)for(blocker=0;blocker<2;blocker++)for(int variant=0;variant<6;variant++){
    e.writeMemory(toAddr(U),new byte[0xc0]);int base=(facing&14)*3/2;int[]frames={0,base,base+1,base+2,base+3,32767};
    put(e,U+0x39,1,facing);put(e,U+0x32,2,frames[variant]);put(e,U+0x80,1,variant*51);put(e,0x622dc0,1,phase);
    run(e,0x459110,0,0);out.println("{\"kind\":\"tick\",\"scenario\":"+scenario+++",\"frame\":"+(short)read(e,U+0x32,2)+",\"remaining\":"+read(e,U+0x80,1)+",\"finished\":"+(finished?1:0)+",\"calls\":"+calls+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(0x460910));put(e,0x632194,4,G);
   for(int scenario=0;scenario<128;scenario++){
    e.writeMemory(toAddr(U),new byte[0xc0]);put(e,U+0x39,1,scenario*13);put(e,U+0x8a,1,scenario*37);random[0]=scenario*2027^0x7731a5c9;random[1]=scenario*7919^0x12348765;
    run(e,0x44cac0,U,0);out.println("{\"kind\":\"reset\",\"scenario\":"+scenario+",\"flags\":"+read(e,U+0x8a,1)+",\"state\":"+read(e,U+0x35,1)+",\"frame\":"+(short)read(e,U+0x32,2)+",\"countdown\":"+read(e,U+0x3d,1)+",\"phase\":"+read(e,U+0x3c,1)+",\"facing\":"+read(e,U+0x40,1)+",\"phase41\":"+read(e,U+0x41,1)+",\"idle\":"+read(e,U+0x7d,1)+",\"calls\":"+randomIndex+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(0x462930));
   for(int x:new int[]{0,1,15,127,128,255,256,511,512,1023,1024,2047,32767,32768,65535}){
    e.writeMemory(toAddr(P),new byte[0x40]);put(e,P,2,x);e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",P);
    if(!e.run(toAddr(0x464000),null,monitor))throw new Exception(e.getLastError());
    if(e.readRegister("EIP").longValue()!=0x462930)throw new Exception("Unexpected prefix stop");
    out.println("{\"kind\":\"bucket\",\"x\":"+x+",\"bucket\":"+(byte)read(e,P+0x2c,1)+"}");
   }
  }finally{e.dispose();}
 }}
}
