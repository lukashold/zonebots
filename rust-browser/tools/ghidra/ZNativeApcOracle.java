// Runs the native automatic APC firing loop. The scan result and final emitter
// are intercepted; seat selection, phase gates and muzzle tables execute as x86.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeApcOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,G=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int direction=0;direction<8;direction++)for(int mask=1;mask<16;mask++)for(int passengers:new int[]{1,6,21}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[0xc0*32]);e.writeMemory(toAddr(G),new byte[0x48]);
     put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,U+0x39,1,direction);put(e,U+0x78,1,80);
     for(int p=0;p<21;p++)put(e,G+0x13+p,1,p<passengers?p+1:255);
     for(long a:new long[]{RET,0x4476f0,0x447790})e.setBreakpoint(toAddr(a));
     e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);
     if(!e.run(toAddr(0x447c20),null,monitor))throw new Exception(e.getLastError());
     int shots=0;
     while(e.readRegister("EIP").longValue()!=RET){
      long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();
      if(a==0x4476f0){int relative=(int)read(e,s+4,4);int side=((relative+8)%8)/2;put(e,e.readRegister("EDX").longValue(),1,(mask&(1<<side))!=0?40+side:255);e.writeRegister("EAX",1);}
      else if(a==0x447790){int x=e.readRegister("ECX").intValue(),y=e.readRegister("EDX").intValue();int side=(int)((read(e,s+8,4)-U)/0xc0)-40;
       out.println("{\"kind\":\"seat\",\"direction\":"+direction+",\"mask\":"+mask+",\"passengers\":"+passengers+",\"shot\":"+shots+",\"side\":"+side+",\"x\":"+x+",\"y\":"+y+"}");shots++;}
      else throw new Exception("Unexpected APC stop");resume(e);
     }
     out.println("{\"kind\":\"burst\",\"direction\":"+direction+",\"mask\":"+mask+",\"passengers\":"+passengers+",\"shots\":"+shots+",\"reload\":"+read(e,U+0x7b,1)+"}");
    }finally{e.dispose();}
   }
   for(int phase=0;phase<16;phase++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[0xc0*32]);e.writeMemory(toAddr(G),new byte[0x48]);
     put(e,U+0x8e,1,15);put(e,U+0x78,1,90);put(e,G+0x13,1,1);for(int p=1;p<21;p++)put(e,G+0x13+p,1,255);put(e,U+0xc0+0xb6,1,phase);
     for(long a:new long[]{RET,0x4476f0,0x447790})e.setBreakpoint(toAddr(a));
     e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);
     if(!e.run(toAddr(0x447c20),null,monitor))throw new Exception(e.getLastError());int shots=0;
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x4476f0){put(e,e.readRegister("EDX").longValue(),1,40);e.writeRegister("EAX",1);}else if(a==0x447790)shots++;else throw new Exception("Unexpected pyro stop");resume(e);}
     out.println("{\"kind\":\"pyro\",\"phase\":"+phase+",\"next\":"+read(e,U+0xc0+0xb6,1)+",\"shots\":"+shots+",\"reload\":"+read(e,U+0x7b,1)+"}");
    }finally{e.dispose();}
   }
  }
 }
}
