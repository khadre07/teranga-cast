(async () => {
  const status = document.getElementById('status');
  const videoEl = document.getElementById('stream');

  function setStatus(msg) {
    status.textContent = msg;
    console.log('[TerangaCast]', msg);
  }

  const pc = new RTCPeerConnection({ iceServers: [] });

  pc.ontrack = (evt) => {
    setStatus('Flux reçu');
    videoEl.srcObject = evt.streams[0];
  };

  pc.oniceconnectionstatechange = () => {
    setStatus('ICE : ' + pc.iceConnectionState);
    if (pc.iceConnectionState === 'connected') {
      status.style.display = 'none';
    }
  };

  try {
    setStatus("Récupération de l'offre…");
    const res = await fetch('/offer');
    const offerData = await res.json();

    await pc.setRemoteDescription(new RTCSessionDescription(offerData));

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    setStatus('Envoi de la réponse…');
    await fetch('/answer', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(pc.localDescription),
    });

    setStatus('En attente du stream…');
  } catch (err) {
    setStatus('Erreur : ' + err.message);
    console.error(err);
  }
})();
