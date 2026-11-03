// Real45f070 order-write stage;43a600 dispersal/classification and43ea80
// spatial lookup are explicit boundaries. Unit/roster writes execute natively.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeOrderEventOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);byte[]group=new byte[72];for(int i=0;i<72;i++)group[i]=(byte)(s*3+i);e.writeMemory(toAddr(R),group);int opcode=s%13+1;
   put(e,U+0x5e,1,new int[]{-1,0,1}[(s/13)%3]);put(e,U+0x5f,1,0);put(e,U+0x69,1,s*7);put(e,U+0x9c,1,s*13);put(e,U+0x6e,1,s);put(e,U+0x50,4,0x1234abcdL+s);put(e,U+0x54,2,-900-s);put(e,U+0x56,2,190+s);
   put(e,0x56ed47,1,opcode);put(e,0x62dc04,4,U);put(e,0x62dc00,2,-200+s*3);put(e,0x62dc3c,2,300-s*7);put(e,0x62dc48,2,-3+s*11);put(e,0x4d02c3,1,1);
   for(long a:new long[]{RET,0x43a600,0x43ea80})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x45f070),null,monitor))throw new Exception(e.getLastError());int steps=0;
   while(e.readRegister("EIP").longValue()!=RET){if(++steps>3)throw new Exception("too many stops");long pc=e.readRegister("EIP").longValue();if(pc==0x43a600){if((s&1)!=0)put(e,0x56ed47,1,2);if((s&2)!=0){put(e,0x62dc00,2,-195+s*3);put(e,0x62dc3c,2,293-s*7);}}else if(pc!=0x43ea80)throw new Exception("Unexpected stop");e.writeRegister("EAX",0);resume(e);}
   out.println("{\"scenario\":"+s+",\"order\":"+(byte)read(e,U+0x5c,1)+",\"flags9c\":"+read(e,U+0x9c,1)+",\"flags69\":"+read(e,U+0x69,1)+",\"counter6e\":"+read(e,U+0x6e,1)+",\"x\":"+(int)read(e,U+0x50,4)+",\"y\":"+(short)read(e,U+0x54,2)+",\"aux\":"+(short)read(e,U+0x56,2)+",\"groupX\":"+(short)read(e,R+44,2)+",\"groupY\":"+(short)read(e,R+46,2)+",\"groupAux\":"+(short)read(e,R+48,2)+",\"groupOrder\":"+(byte)read(e,R+58,1)+"}");
  }finally{e.dispose();}
 }}}
}
