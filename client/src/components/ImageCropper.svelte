<script lang="ts">
  import Cropper from "svelte-easy-crop";

  let {
    aspect = 1,
    crop,
    height = 0,
    imageDataUrl = "",
    transformations = {},
    width = 0,
    zoom = 1,
  } = $props();

  function setImgSrc(node: PointerEvent) {
    const cropperImage: HTMLImageElement = node.getElementsByClassName("svelte-easy-crop-image")[0];
    cropperImage.setAttribute("src", imageDataUrl);
  }

  function onCropComplete(details: object) {
    const cropDetails: object = details.pixels;
    if (cropDetails) {
      width = cropDetails.width;
      height = cropDetails.height;
      transformations.crop = cropDetails;
    }
  }
</script>

<div class="cropper-container" use:setImgSrc>
  <Cropper
    {imageDataUrl}
    bind:crop
    bind:zoom
    aspect={aspect}
    oncropcomplete={onCropComplete}
  />
</div>

<style>
  .cropper-container {
    position: relative;
    width: 100%;
    height: 75%;
  }

  @media (max-width: 640px) {
    .cropper-container {
      min-height: 50vh;
    }
  }
</style>
