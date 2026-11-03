// Execute actual4b280 and4b220 over complete240-group pools and sparse roles.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeGroupCountersOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(int h,int n){return(h^n)*16777619;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<32;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[240*192]);
    for(int g=0;g<240;g++){
     byte[]r=new byte[72];for(int j=0;j<72;j++)r[j]=(byte)(scenario*7+g*3+j);for(int role=0;role<21;role++)r[19+role]=(byte)255;
     r[9]=(byte)((g+scenario)%2==0?64:5);r[16]=(byte)((g+scenario)%4);r[18]=0;
     if(g<8)for(int role=0;role<21;role++)if(role==0||(role+scenario+g)%5>=2){int id=g*21+role;r[19+role]=(byte)id;r[18]++;put(e,U+id*192+0x5e,1,role==0?255:role+(scenario%3==0?3:0));}
     e.writeMemory(toAddr(R+g*72),r);
    }
    e.setBreakpoint(toAddr(RET));
    for(int tick=0;tick<4;tick++){
     e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x44b280),null,monitor))throw new Exception(e.getLastError());
     int rh=0x811c9dc5,mh=0x811c9dc5;for(int i=0;i<240*72;i++)rh=hash(rh,(int)read(e,R+i,1));for(int i=0;i<240;i++)mh=hash(mh,(byte)read(e,U+i*192+0x5e,1));
     out.println("{\"scenario\":"+scenario+",\"tick\":"+tick+",\"busy\":"+read(e,0x5b6d92,2)+",\"roster\":"+Integer.toUnsignedString(rh)+",\"roles\":"+Integer.toUnsignedString(mh)+"}");
    }
   }finally{e.dispose();}
  }
 }}
}
