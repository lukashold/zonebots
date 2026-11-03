// Complete local N/S/E/W scheduling, stopped before route adoption/world actions.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeLocalFallbackOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,P=U+0x2000,F=U+0x3000,M=U+0x4000,C=0x4d15f0L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);put(e,S+4,4,F);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<100000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET||pc==0x43e190)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  int[][]starts={{464,512,4126,-1},{560,512,4130,-1},{512,304,-1,2592},{512,656,-1,5152}};int[][]goals={{648,520},{648,648},{168,520}};
  for(int vehicle=1;vehicle<2;vehicle++)for(int previous=0;previous<2;previous++)for(int si=0;si<4;si++)for(int gi=0;gi<3;gi++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[0x9000]);e.writeMemory(toAddr(C),new byte[400]);byte[]map=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++)if(x>=30&&x<=34&&y>=20&&y<=40)map[y*128+x]=1;e.writeMemory(toAddr(0x58aaaa),map);put(e,T+1,1,255);put(e,U+0x18,4,T);put(e,U+0x14,4,P);put(e,U+0x7c,1,vehicle);put(e,U+0x31,1,68);put(e,U+0xb4,1,previous);put(e,U+0x2c,2,starts[si][0]);put(e,U+0x2e,2,starts[si][1]);put(e,U+0x58,2,starts[si][2]);put(e,U+0x5a,2,starts[si][3]);put(e,U+0x10,2,((starts[si][0]+8)>>4)+(((starts[si][1]+8)>>4)*128));put(e,P,2,goals[gi][0]);put(e,P+2,2,goals[gi][1]);put(e,0x63288c,4,U);put(e,0x633948,4,M);put(e,0x4d12f8,2,0);put(e,0x586862,1,127);put(e,0x6338dc,4,0x43e4a0);
    int callbacks=0;while(++callbacks<=200){call(e,read(e,0x6338dc,4));if(e.readRegister("EIP").longValue()==0x43e190)break;}if(callbacks>200)throw new Exception("Local callback budget");call(e,0x43dcf0);callbacks=0;while(++callbacks<=200){call(e,read(e,0x6338dc,4));if(e.readRegister("EIP").longValue()==0x43e190)break;}if(callbacks>200)throw new Exception("Fallback callback budget");StringBuilder line=new StringBuilder(vehicle+","+previous+","+si+","+gi+","+read(e,0x4d15e4,2)+","+read(e,U+0xb4,1));
    for(int d=0;d<4;d++){long h=C+d*100;int count=(byte)read(e,h+1,1);line.append(',').append(read(e,h,1)).append(',').append(count);for(int j=0;j<Math.abs(count)&&j<48;j++)line.append(',').append((short)read(e,h+2+j*2,2));line.append(";");}out.println(line);out.flush();
   }finally{e.dispose();}
  }
 }}
}
